<script lang="ts">
  // 状態管理
  import { invoke } from "@tauri-apps/api/tauri";
  import { enable, isEnabled, disable } from "tauri-plugin-autostart-api";
  import { onMount } from "svelte";
  import { info, attachConsole } from "tauri-plugin-log-api";
  let isDarkMode = true;
  let isAutoStartEnabled = false;

  onMount(async () => {
    setTimeout(() => {
      setupWindow();
    }, 300);
    document.documentElement.classList.toggle("dark", isDarkMode);
  });
  $: isEnabled().then((isEnabled: boolean) => (isAutoStartEnabled = isEnabled));
  async function setupWindow() {
    const appWindow = (await import("@tauri-apps/api/window")).appWindow;
    appWindow.show();
  }

  // トグル動作のハンドリング
  function toggleAutoStart() {
    isAutoStartEnabled = !isAutoStartEnabled;
    isAutoStartEnabled ? enable() : disable();
  }

  /* function toggleTheme() {
    isDarkMode = !isDarkMode;
    document.documentElement.classList.toggle("dark", isDarkMode);
  } */
</script>

<!-- 設定画面 -->
<main>
  <div
    class="p-6 max-w-md mx-auto bg-white dark:bg-gray-800 rounded-xl shadow-md space-y-4"
  >
    <h2 class="text-xl font-bold text-gray-900 dark:text-white">Settings</h2>

    <!-- AutoStartトグル -->
    <span>
      <div class="flex items-center">
        <label
          for="autoStartToggle"
          class="flex-grow text-gray-900 dark:text-white">Auto Start</label
        >
        <button
          class={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
            isAutoStartEnabled ? "bg-blue-500" : "bg-gray-500"
          }`}
          on:click={toggleAutoStart}
        >
          <span
            class={`transform transition-transform duration-300 ease-in-out ${
              isAutoStartEnabled ? "translate-x-6" : "translate-x-1"
            } inline-block h-4 w-4 rounded-full bg-white`}
          />
        </button>
      </div>

      <!-- Themeトグル -->
    </span>
  </div>
</main>

<style>
  :global(.dark) {
    --tw-bg-opacity: 1;
    background-color: rgba(31, 41, 55, var(--tw-bg-opacity));
  }
</style>
