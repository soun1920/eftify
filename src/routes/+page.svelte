<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { emit, listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/api/dialog";
  import { onMount } from "svelte";

  let path = "";
  let isSpotifyRunning = false;
  onMount(async () => {
    await invoke<string>("start_loop");
    listen<boolean>("back-to-front", (event) => {
      isSpotifyRunning = event.payload;
      console.log("Received event", event.payload);
    });

    try {
      path = await invoke<string>("simple_command");
    } catch (e) {
      console.error("Failed to fetch message from backend:", e);
    }
  });
  let name = "";
  let greetMsg = "";
  function openDialog() {
    open().then((files) => console.log(files));
  }
  function spotifyPlay() {
    invoke("spotify_play").then((message) => {
      console.log("spotify_play", message);
    });
  }
  function spotifyPause() {
    invoke("spotify_pause").then((message) => {
      console.log("spotify_play", message);
    });
  }
  function executeCommands() {
    invoke("command_with_message", { message: "some message" }).then(
      (message) => {
        console.log("command_with_messge", message);
      }
    );
  }
</script>

<button on:click={spotifyPlay}>Play</button>
<button on:click={spotifyPause}>Pause</button>
<div class="container">
  <p>{isSpotifyRunning}</p>
</div>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: 0.75s;
  }

  .logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  .row {
    display: flex;
    justify-content: center;
  }

  a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
  }

  a:hover {
    color: #535bf2;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    a:hover {
      color: #24c8db;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
