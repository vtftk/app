<script lang="ts">
  import type { Component } from "svelte";
  import type { SvelteHTMLElements } from "svelte/elements";

  import { page } from "$app/state";

  type Props = {
    icon: Component<SvelteHTMLElements["svg"]>;
    /** Expects resolve() to be used before passing href in */
    href: string;
    title: string;
    text: string;
  };

  const { icon: Icon, href, title, text }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -->

<a
  {href}
  class="button"
  class:button--selected={href === "/"
    ? page.route.id === href
    : page.route.id?.startsWith(href)}
>
  <Icon class="icon" />
  <div class="content">
    <p class="title">{title}</p>
    <span class="text">{text}</span>
  </div>
</a>

<style>
  .button {
    display: block;
    padding: 0.5rem;
    border: 1px solid #2f2f2f;
    text-decoration: none;
    background-color: #222;
    border-radius: 0.5rem;
    display: flex;
    gap: 0.75rem;
    align-items: center;
    transition:
      background-color 0.25s ease,
      transform 0.15s ease;
  }

  .button:active {
    transform: scale(0.98);
  }

  .content {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .button:global(> .icon) {
    color: white;
    width: 1.5rem;
    height: 1.5rem;
  }

  .button:hover {
    background-color: #333;
  }

  .button--selected {
    background-color: #444;
  }

  .button--selected:hover {
    background-color: #555;
  }

  .title {
    color: #fff;
    line-height: 1;
  }

  .text {
    color: #ccc;
    font-size: 0.8rem;
    line-height: 1;
  }
</style>
