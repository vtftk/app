<script lang="ts">
  import { resolve } from "$app/paths";
  import HomeBoldDuotone from "~icons/solar/home-bold-duotone";
  import SolarBellBoldDuotone from "~icons/solar/bell-bold-duotone";
  import SolarSettingsBoldDuotone from "~icons/solar/settings-bold-duotone";
  import SolarBasketballBoldDuotone from "~icons/solar/basketball-bold-duotone";
  import SolarChatSquareCodeBoldDuotone from "~icons/solar/chat-square-code-bold-duotone";
  import SolarHeadphonesRoundSoundBoldDuotone from "~icons/solar/headphones-round-sound-bold-duotone";

  import NavButton from "./NavButton.svelte";
</script>

<nav>
  <ul>
    <li>
      <NavButton
        icon={HomeBoldDuotone}
        href={resolve("/")}
        title="Home"
        text="General details"
      />
    </li>
    <li>
      <NavButton
        icon={SolarBasketballBoldDuotone}
        href={resolve("/throwables")}
        title="Throwables"
        text="Items that can be thrown"
      />
    </li>
    <li>
      <NavButton
        icon={SolarBellBoldDuotone}
        href={resolve("/events")}
        title="Events"
        text="Events that can trigger actions"
      />
    </li>
    <li>
      <NavButton
        icon={SolarHeadphonesRoundSoundBoldDuotone}
        href={resolve("/sounds")}
        title="Sounds"
        text="Sounds that can be triggered"
      />
    </li>
    <li>
      <NavButton
        icon={SolarChatSquareCodeBoldDuotone}
        href={resolve("/commands")}
        title="Commands"
        text="Custom commands"
      />
    </li>
    <li>
      <NavButton
        icon={SolarSettingsBoldDuotone}
        href={resolve("/settings")}
        title="Settings"
        text="Configuration options"
      />
    </li>
  </ul>
</nav>

<style>
  nav > ul {
    display: flex;
    flex-flow: column;
    gap: 0.5em;
  }
</style>
