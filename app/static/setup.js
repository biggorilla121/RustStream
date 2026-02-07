(function () {
  const input = document.getElementById('tmdb');
  const button = document.getElementById('save');
  const status = document.getElementById('status');

  function getInvoke() {
    if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {
      return window.__TAURI__.invoke;
    }
    if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {
      return window.__TAURI__.tauri.invoke;
    }
    return null;
  }

  async function saveKey() {
    const key = (input.value || '').trim();
    if (!key) {
      status.textContent = 'Please enter a token.';
      return;
    }

    const invoke = getInvoke();
    if (!invoke) {
      status.textContent = 'Tauri API not available.';
      return;
    }

    button.disabled = true;
    status.textContent = 'Saving...';

    try {
      await invoke('save_tmdb_key', { key });
      status.textContent = 'Saved. Launching RustStream...';
    } catch (err) {
      status.textContent = 'Error: ' + (err?.toString?.() || err);
      button.disabled = false;
    }
  }

  button.addEventListener('click', saveKey);
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      saveKey();
    }
  });
})();
