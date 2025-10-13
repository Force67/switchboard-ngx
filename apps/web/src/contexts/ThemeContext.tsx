import { createContext, useContext, createSignal, Accessor, onMount } from 'solid-js';

export type Theme = 'light' | 'dark' | 'system';

interface ThemeContextType {
  theme: Accessor<Theme>;
  effectiveTheme: Accessor<'light' | 'dark'>;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  isSystem: Accessor<boolean>;
}

const ThemeContext = createContext<ThemeContextType>();

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

const THEME_STORAGE_KEY = 'switchboard-theme';

export function ThemeProvider(props: { children: JSX.Element }) {
  const [theme, setThemeSignal] = createSignal<Theme>('system');
  const [effectiveTheme, setEffectiveTheme] = createSignal<'light' | 'dark'>('dark');

  const isSystem = () => theme() === 'system';

  const updateEffectiveTheme = (newTheme: Theme) => {
    if (newTheme === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      setEffectiveTheme(systemTheme);
    } else {
      setEffectiveTheme(newTheme as 'light' | 'dark');
    }
  };

  const setTheme = (newTheme: Theme) => {
    setThemeSignal(newTheme);
    localStorage.setItem(THEME_STORAGE_KEY, newTheme);
    updateEffectiveTheme(newTheme);
    updateCSSVariables(newTheme);
  };

  const toggleTheme = () => {
    const current = effectiveTheme();
    setTheme(current === 'dark' ? 'light' : 'dark');
  };

  const updateCSSVariables = (themeValue: Theme) => {
    const root = document.documentElement;

    if (themeValue === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      applyTheme(systemTheme);
    } else {
      applyTheme(themeValue as 'light' | 'dark');
    }
  };

  const applyTheme = (themeToApply: 'light' | 'dark') => {
    const root = document.documentElement;

    if (themeToApply === 'dark') {
      root.setAttribute('data-theme', 'dark');
    } else {
      root.setAttribute('data-theme', 'light');
    }
  };

  onMount(() => {
    // Load saved theme
    const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as Theme;
    if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
      setTheme(savedTheme);
    } else {
      setTheme('system');
    }

    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemThemeChange = () => {
      if (isSystem()) {
        updateEffectiveTheme('system');
        updateCSSVariables('system');
      }
    };

    mediaQuery.addEventListener('change', handleSystemThemeChange);

    return () => {
      mediaQuery.removeEventListener('change', handleSystemThemeChange);
    };
  });

  const value: ThemeContextType = {
    theme,
    effectiveTheme,
    setTheme,
    toggleTheme,
    isSystem,
  };

  return (
    <ThemeContext.Provider value={value}>
      {props.children}
    </ThemeContext.Provider>
  );
}