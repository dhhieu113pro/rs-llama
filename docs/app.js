(() => {
  const root = document.documentElement;
  const toggle = document.getElementById('theme-toggle');
  const status = document.getElementById('copy-status');
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const storageKey = 'site-theme';
  const modes = ['auto', 'light', 'dark'];

  const currentMode = () => root.dataset.theme || 'auto';

  const renderTheme = () => {
    if (!toggle) return;
    const mode = currentMode();
    const label = mode === 'auto' ? 'Auto' : mode === 'light' ? 'Light' : 'Dark';
    toggle.textContent = label;
    toggle.dataset.themeMode = mode;
    toggle.title = `Theme: ${label} — click to change`;
    toggle.setAttribute('aria-label', `Change color theme. Current: ${label}`);
  };

  const applyTheme = (mode) => {
    try {
      if (mode === 'auto') {
        delete root.dataset.theme;
        localStorage.removeItem(storageKey);
      } else {
        root.dataset.theme = mode;
        localStorage.setItem(storageKey, mode);
      }
    } catch {
      if (mode === 'auto') delete root.dataset.theme;
      else root.dataset.theme = mode;
    }
    renderTheme();
  };

  try {
    const saved = localStorage.getItem(storageKey);
    if (saved === 'light' || saved === 'dark') root.dataset.theme = saved;
  } catch {
    // Storage can be unavailable in hardened/private contexts; system theme still works.
  }
  renderTheme();

  toggle?.addEventListener('click', () => {
    const current = currentMode();
    applyTheme(modes[(modes.indexOf(current) + 1) % modes.length]);
  });

  media.addEventListener?.('change', () => {
    if (currentMode() === 'auto') renderTheme();
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
