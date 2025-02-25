use crate::{
    database::{
        entity::{
            command_execution::{
                CommandExecutionMetadata, CommandExecutionModel, CreateCommandExecution,
            },
            commands::{CommandModel, CommandOutcome},
            event_execution::{CreateEventExecution, EventExecutionMetadata, EventExecutionModel},
            events::EventModel,
            shared::MinimumRequireRole,
        },
        DbPool,
    },
    events::{
        matching::{
            match_ad_break_event, match_chat_event, match_cheer_bits_event, match_follow_event,
            match_gifted_subscription_event, match_raid_event, match_re_subscription_event,
            match_redeem_event, match_shoutout_receive_event, match_subscription_event,
            match_timer_complete_event, CommandWithContext, EventData, EventInputData,
            EventMatchingData,
        },
        outcome::produce_outcome_message,
        AppEvent, TwitchEventUser,
    },
    overlay::OverlayMessageSender,
    script::runtime::{
        CommandContext, CommandContextUser, RuntimeExecutionContext, ScriptExecutorHandle,
    },
    twitch::manager::Twitch,
};
use anyhow::{anyhow, Context};
use chrono::{DateTime, TimeDelta, Utc};
use futures::{future::BoxFuture, stream::FuturesUnordered, Stream};
use log::{debug, error};
use std::{future::poll_fn, task::Poll, time::Duration};
use tauri::{AppHandle, Emitter};
use tokio::try_join;
use twitch_api::types::UserId;

use super::AppEventReceiver;

pub async fn process_events(
    db: DbPool,
    twitch: Twitch,
    script_handle: ScriptExecutorHandle,
    event_sender: OverlayMessageSender,
    app_handle: AppHandle,

    mut event_rx: AppEventReceiver,
) {
    let mut futures = FuturesUnordered::new();
    let mut futures = std::pin::pin!(futures);

    poll_fn::<(), _>(|cx| {
        // Poll new event execution
        while let Poll::Ready(result) = event_rx.poll_recv(cx) {
            let event = match result {
                Some(value) => value,

                // All channels have been closed and the app is likely shutting down,
                // finish the future and stop processing
                None => return Poll::Ready(()),
            };

            debug!("app event received: {:?}", event);

            futures.push(process_event(
                &db,
                &twitch,
                &script_handle,
                &event_sender,
                &app_handle,
                event,
            ));
        }

        // Poll completions until no more ready
        while let Poll::Ready(Some(result)) = futures.as_mut().poll_next(cx) {
            if let Err(err) = result {
                debug!("failed to process app event: {err:?}",);
            }
        }

        Poll::Pending
    })
    .await;
}

async fn process_event(
    db: &DbPool,
    twitch: &Twitch,
    script_handle: &ScriptExecutorHandle,
    event_sender: &OverlayMessageSender,
    app_handle: &AppHandle,
    event: AppEvent,
) -> anyhow::Result<()> {
    let match_data: EventMatchingData = match event {
        // Matchable events
        AppEvent::Redeem(event) => match_redeem_event(db, event).await?,
        AppEvent::CheerBits(event) => match_cheer_bits_event(db, event).await?,
        AppEvent::Follow(event) => match_follow_event(db, event).await?,
        AppEvent::Sub(event) => match_subscription_event(db, event).await?,
        AppEvent::GiftSub(event) => match_gifted_subscription_event(db, event).await?,
        AppEvent::ResubMsg(event) => match_re_subscription_event(db, event).await?,
        AppEvent::ChatMsg(event) => match_chat_event(db, event).await?,
        AppEvent::Raid(event) => match_raid_event(db, event).await?,
        AppEvent::AdBreakBegin(event) => match_ad_break_event(db, event).await?,
        AppEvent::ShoutoutReceive(event) => match_shoutout_receive_event(db, event).await?,
        AppEvent::TimerCompleted(event) => match_timer_complete_event(db, twitch, event).await?,

        // Internal events
        AppEvent::ModeratorsChanged => {
            debug!("reloading mods list");
            twitch.load_moderator_list().await?;
            return Ok(());
        }
        AppEvent::VipsChanged => {
            debug!("reloading vips list");
            twitch.load_vip_list().await?;
            return Ok(());
        }
        AppEvent::RewardsChanged => {
            debug!("reloading rewards list");
            twitch.load_rewards_list().await?;
            return Ok(());
        }
        AppEvent::TwitchClientReset => {
            debug!("resetting twitch manager");
            twitch.reset().await;
            return Ok(());
        }

        AppEvent::TwitchClientLoggedOut => {
            _ = app_handle.emit("logout", ());
            return Ok(());
        }
        AppEvent::TwitchClientLoggedIn => {
            _ = app_handle.emit("authenticated", ());
            return Ok(());
        }
    };

    let command_futures =
        match_data
            .commands
            .into_iter()
            .map(|command| -> BoxFuture<'_, anyhow::Result<()>> {
                Box::pin(execute_command(
                    db,
                    script_handle,
                    twitch,
                    command,
                    &match_data.event_data,
                ))
            });

    let event_futures =
        match_data
            .events
            .into_iter()
            .map(|event| -> BoxFuture<'_, anyhow::Result<()>> {
                Box::pin(execute_event(
                    db,
                    twitch,
                    script_handle,
                    event_sender,
                    event,
                    &match_data.event_data,
                ))
            });

    let mut futures = command_futures
        .chain(event_futures)
        .collect::<FuturesUnordered<BoxFuture<'_, anyhow::Result<()>>>>();

    use futures::StreamExt;

    while let Some(result) = futures.next().await {
        if let Err(err) = result {
            error!("error while executing event outcome: {err:?}");
        }
    }

    Ok(())
}

pub fn is_cooldown_elapsed(
    execution_time: DateTime<Utc>,
    current_time: DateTime<Utc>,
    cooldown: u32,
) -> anyhow::Result<bool> {
    let cooldown_end_time = execution_time
        .checked_add_signed(TimeDelta::milliseconds(cooldown as i64))
        .context("cooldown finishes too far in the future to compute")?;

    Ok(current_time > cooldown_end_time)
}

pub async fn is_command_cooldown_elapsed(
    db: &DbPool,
    command: &CommandModel,
    user_id: &UserId,
    current_time: DateTime<Utc>,
) -> anyhow::Result<bool> {
    let cooldown = &command.config.cooldown;

    // No cooldown enabled
    if !cooldown.enabled {
        return Ok(true);
    }

    // Handle global cooldown (Check last execution)
    if !cooldown.per_user {
        let last_execution = CommandExecutionModel::last(db, command.id, 0)
            .await
            .context("failed to request last execution for command")?;

        let last_execution = match last_execution {
            Some(value) => value,
            None => return Ok(true),
        };

        return is_cooldown_elapsed(last_execution.created_at, current_time, cooldown.duration);
    }

    let mut offset = 0;

    loop {
        let last_execution = CommandExecutionModel::last(db, command.id, offset)
            .await
            .context("failed to request last execution for command")?;

        // Cooldown elapsed if no more to check
        let last_execution = match last_execution {
            Some(value) => value,
            None => return Ok(true),
        };

        // Check if the execution has elapsed the cooldown
        let last_execution_elapsed =
            is_cooldown_elapsed(last_execution.created_at, current_time, cooldown.duration)?;

        // Execution was not from the target user
        if last_execution
            .metadata
            .user
            .is_none_or(|event_user| event_user.id != user_id)
        {
            offset += 1;

            // If the execution had elapsed the cooldown we can treat the rest as elapsed too (We can skip checking the rest)
            if last_execution_elapsed {
                return Ok(true);
            }

            continue;
        }

        // Found an execution
        return Ok(last_execution_elapsed);
    }
}

pub async fn execute_command(
    db: &DbPool,
    script_handle: &ScriptExecutorHandle,
    twitch: &Twitch,
    command: CommandWithContext,
    event_data: &EventData,
) -> anyhow::Result<()> {
    let EventInputData::Chat {
        message,
        message_id,
        ..
    } = &event_data.input_data
    else {
        return Err(anyhow!("Non chat input data provided for chat execute"));
    };

    let user = match event_data.user.clone() {
        Some(value) => value,
        None => return Err(anyhow!("got chat event without a user")),
    };

    // Ensure required role is present
    if !has_required_role(twitch, Some(&user.id), &command.command.config.require_role).await {
        debug!("skipping command: missing required role");
        return Ok(());
    }

    let current_time = Utc::now();

    if !is_command_cooldown_elapsed(db, &command.command, &user.id, current_time).await? {
        debug!("skipping command: cooldown");
        return Ok(());
    }

    // Create metadata for storage
    let metadata = CommandExecutionMetadata {
        user: Some(user.clone()),
        data: vec![(
            "input_data".to_string(),
            serde_json::to_value(&event_data.input_data)
                .context("failed to serialize command metadata")?,
        )],
    };

    match command.command.config.outcome {
        CommandOutcome::Template { message } => {
            let to_usr = command
                .args
                .first()
                .map(|value| value.as_str())
                .unwrap_or_default();
            let message = message
                .replace("$(user)", user.name.as_str())
                .replace("$(touser)", to_usr);

            twitch.send_chat_message_chunked(&message).await?;
        }
        CommandOutcome::Script { script } => {
            let user = CommandContextUser {
                id: user.id,
                name: user.name,
                display_name: user.display_name,
            };

            let ctx = CommandContext {
                message_id: message_id.to_string(),
                full_message: message.to_string(),
                input_data: event_data.input_data.clone(),
                message: command.message,
                args: command.args,
                user,
            };

            script_handle
                .execute_command(
                    RuntimeExecutionContext::Command {
                        command_id: command.command.id,
                    },
                    script,
                    ctx,
                )
                .await?;
        }
    }

    // Store command execution
    CommandExecutionModel::create(
        db,
        CreateCommandExecution {
            command_id: command.command.id,
            created_at: current_time,
            metadata,
        },
    )
    .await
    .context("failed to store last command execution")?;

    Ok(())
}

pub async fn is_event_cooldown_elapsed(
    db: &DbPool,
    event: &EventModel,
    user: Option<&TwitchEventUser>,
    current_time: DateTime<Utc>,
) -> anyhow::Result<bool> {
    let cooldown = &event.config.cooldown;

    // No cooldown enabled
    if !cooldown.enabled {
        return Ok(true);
    }

    // Handle global cooldown (Check last execution)
    if !cooldown.per_user {
        let last_execution = EventExecutionModel::last(db, event.id, 0)
            .await
            .context("failed to request last execution for event")?;

        let last_execution = match last_execution {
            Some(value) => value,
            None => return Ok(true),
        };

        return is_cooldown_elapsed(last_execution.created_at, current_time, cooldown.duration);
    }

    let user = match user {
        Some(user) => user,
        // Anonymous users bypass the cooldown
        None => return Ok(true),
    };

    let mut offset = 0;

    loop {
        let last_execution = EventExecutionModel::last(db, event.id, offset)
            .await
            .context("failed to request last execution for event")?;

        // Cooldown elapsed if no more to check
        let last_execution = match last_execution {
            Some(value) => value,
            None => return Ok(true),
        };

        // Check if the execution has elapsed the cooldown
        let last_execution_elapsed =
            is_cooldown_elapsed(last_execution.created_at, current_time, cooldown.duration)?;

        // Execution was not from the target user
        if last_execution
            .metadata
            .user
            .is_none_or(|event_user| event_user.id != user.id)
        {
            offset += 1;

            // If the execution had elapsed the cooldown we can treat the rest as elapsed too (We can skip checking the rest)
            if last_execution_elapsed {
                return Ok(true);
            }

            continue;
        }

        // Found an execution
        return Ok(last_execution_elapsed);
    }
}

pub async fn execute_event(
    db: &DbPool,
    twitch: &Twitch,
    script_handle: &ScriptExecutorHandle,

    event_sender: &OverlayMessageSender,
    event: EventModel,
    event_data: &EventData,
) -> anyhow::Result<()> {
    // Ensure required role is present
    if !has_required_role(
        twitch,
        event_data.user.as_ref().map(|value| &value.id),
        &event.config.require_role,
    )
    .await
    {
        debug!("skipping event: missing required role");
        return Ok(());
    }

    let current_time = Utc::now();

    // Ensure cooldown is not active
    if !is_event_cooldown_elapsed(db, &event, event_data.user.as_ref(), current_time).await? {
        debug!("skipping event: cooldown");
        return Ok(());
    }

    // Create metadata for storage
    let metadata = EventExecutionMetadata {
        user: event_data.user.clone(),
        data: vec![(
            "input_data".to_string(),
            serde_json::to_value(&event_data.input_data)
                .context("failed to serialize event metadata")?,
        )],
    };

    // Wait for outcome delay
    tokio::time::sleep(Duration::from_millis(event.config.outcome_delay as u64)).await;

    let event_id = event.id;

    // Produce outcome message and send it
    if let Some(msg) = produce_outcome_message(db, twitch, script_handle, event, event_data).await?
    {
        _ = event_sender.send(msg);
    }

    // Store event execution
    EventExecutionModel::create(
        db,
        CreateEventExecution {
            event_id,
            created_at: current_time,
            metadata,
        },
    )
    .await
    .context("failed to store last event execution")?;

    Ok(())
}

pub async fn has_required_role(
    twitch: &Twitch,
    user_id: Option<&UserId>,
    required_role: &MinimumRequireRole,
) -> bool {
    // No role required (Initial level)
    if let MinimumRequireRole::None = required_role {
        return true;
    }

    // User must be authenticated at this point
    let user = match user_id {
        Some(user) => user,
        None => return false,
    };

    // User is the broadcaster
    if twitch
        .get_user_token()
        .await
        .is_some_and(|value| value.user_id == user)
    {
        return true;
    }

    // Check VIP and moderator lists
    if let MinimumRequireRole::Vip = required_role {
        return try_join!(twitch.get_vip_list(), twitch.get_moderator_list()).is_ok_and(
            |(vips, mods)| {
                vips.iter().any(|vip| vip.user_id == user)
                    || mods.iter().any(|mods| mods.user_id == user)
            },
        );
    }

    // Check just moderator list
    if let MinimumRequireRole::Mod = required_role {
        return twitch
            .get_moderator_list()
            .await
            .is_ok_and(|mods| mods.iter().any(|mods| mods.user_id == user));
    }

    // Check the user is a follower
    if let MinimumRequireRole::Follower = required_role {
        return twitch
            .get_follower_by_id(user)
            .await
            .is_ok_and(|value| value.is_some());
    }

    false
}
