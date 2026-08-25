(() => {
  const root = document.documentElement;
  const toggle = document.getElementById('theme-toggle');
  const icon = document.getElementById('theme-icon');
  const status = document.getElementById('copy-status');
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const storageKey = 'rs-llama-theme';

  const effectiveTheme = () => root.dataset.theme || (media.matches ? 'dark' : 'light');

  const renderTheme = () => {
    const theme = effectiveTheme();
    if (icon) icon.textContent = theme === 'dark' ? '☀' : '◐';
    if (toggle) {
      toggle.title = theme === 'dark' ? 'Use light theme' : 'Use dark theme';
      toggle.setAttribute('aria-pressed', theme === 'dark' ? 'true' : 'false');
    }
  };

  try {
    const saved = localStorage.getItem(storageKey);
    if (saved === 'light' || saved === 'dark') root.dataset.theme = saved;
  } catch {
    // Storage can be unavailable in hardened/private contexts; system theme still works.
  }
  renderTheme();

  toggle?.addEventListener('click', () => {
    const next = effectiveTheme() === 'dark' ? 'light' : 'dark';
    root.dataset.theme = next;
    try {
      localStorage.setItem(storageKey, next);
    } catch {
      // Keep the in-memory override even when persistence is unavailable.
    }
    renderTheme();
  });

  media.addEventListener?.('change', () => {
    if (!root.dataset.theme) renderTheme();
  });

  document.querySelectorAll('[data-copy-target]').forEach((button) => {
    button.dataset.state = 'default';
    button.addEventListener('click', async () => {
      const target = document.getElementById(button.dataset.copyTarget);
      const original = button.textContent;

      if (!target || !navigator.clipboard) {
        button.dataset.state = 'error';
        button.textContent = 'Select';
        if (status) status.textContent = 'Clipboard access is unavailable. Select the code manually.';
        window.setTimeout(() => {
          button.dataset.state = 'default';
          button.textContent = original;
        }, 1400);
        return;
      }

      button.dataset.state = 'loading';
      try {
        await navigator.clipboard.writeText(target.textContent);
        button.dataset.state = 'success';
        button.textContent = 'Copied';
        if (status) status.textContent = 'Code copied to clipboard.';
      } catch {
        button.dataset.state = 'error';
        button.textContent = 'Retry';
        if (status) status.textContent = 'Copy failed. Select the code manually.';
      }

      window.setTimeout(() => {
        button.dataset.state = 'default';
        button.textContent = original;
      }, 1400);
    });
  });
})();
