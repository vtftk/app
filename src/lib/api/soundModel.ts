import type {
  Sound,
  SoundId,
  CreateSound,
  UpdateSound,
  UpdateOrdering,
} from "$lib/api/types";

import { invoke } from "@tauri-apps/api/core";
import { createQuery } from "@tanstack/svelte-query";

import { queryClient } from "./client";

const SOUNDS_KEY = ["sounds"];

function createSoundKey(id: SoundId) {
  return ["sound", id] as const;
}

// -----------------------------------------------------

function invalidateSoundsList() {
  // Invalid the list of sounds
  queryClient.invalidateQueries({ queryKey: SOUNDS_KEY });
}

export function getSoundById(soundId: SoundId) {
  return invoke<Sound | null>("get_sound_by_id", { soundId });
}

export async function createSound(create: CreateSound, invalidateList = true) {
  const sound = await invoke<Sound>("create_sound", { create });

  // Invalidate the specific sound query
  const soundKey = createSoundKey(sound.id);
  queryClient.setQueryData(soundKey, sound);

  if (invalidateList) invalidateSoundsList();

  return sound;
}

export async function createSounds(creates: CreateSound[]) {
  await Promise.all(creates.map((create) => createSound(create, false)));

  invalidateSoundsList();
}

export async function updateSound(update: UpdateSound, invalidateList = true) {
  const sound = await invoke<Sound>("update_sound", update);
  // Invalidate the specific item query
  const itemKey = createSoundKey(sound.id);
  queryClient.setQueryData(itemKey, sound);

  if (invalidateList) invalidateSoundsList();
}

export async function deleteSound(soundId: SoundId, invalidateList = true) {
  await invoke<void>("delete_sound", { soundId });

  const soundKey = createSoundKey(soundId);

  // Cancel any queries for the item and clear the current item data
  queryClient.cancelQueries({ queryKey: soundKey });
  queryClient.setQueryData(soundKey, undefined);

  if (invalidateList) invalidateSoundsList();
}

export async function deleteSounds(soundIds: SoundId[]) {
  await Promise.all(soundIds.map((soundId) => deleteSound(soundId, false)));
  invalidateSoundsList();
}

export async function updateSoundOrder(update: UpdateOrdering[]) {
  await invoke("update_sound_orderings", { update });

  invalidateSoundsList();
}

// -----------------------------------------------------

export function createSoundsQuery() {
  return createQuery(() => ({
    queryKey: SOUNDS_KEY,
    queryFn: () => invoke<Sound[]>("get_sounds"),
  }));
}

export function createSoundQuery(idParam: () => SoundId) {
  return createQuery(() => {
    const id = idParam();

    return {
      queryKey: createSoundKey(id),
      queryFn: () => getSoundById(id),
    };
  });
}
