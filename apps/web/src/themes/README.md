# Theme System

This directory contains the global theme system for Switchboard NGX.

## Overview

The theme system provides:
- **Light theme**: Clean, bright interface with blue accents
- **Dark theme**: Dark interface with pink/purple accents (original theme)
- **System theme**: Automatically follows user's OS preference

## Files

- `theme-config.css` - Contains all color definitions and CSS variables for both themes
- `ThemeContext.tsx` - SolidJS context providing theme state and management
- `ThemeToggle.tsx` - Reusable theme toggle button component

## Usage

### Using the Theme Context

```tsx
import { useTheme } from '../contexts/ThemeContext';

function MyComponent() {
  const { effectiveTheme, setTheme, toggleTheme } = useTheme();

  const isDark = effectiveTheme() === 'dark';
  // ...
}
```

### Theme-aware CSS

All colors use CSS custom properties that automatically update based on the current theme:

```css
.my-component {
  background: var(--bg-1);
  color: var(--text-0);
  border: 1px solid var(--hair);
}
```

### Available Color Variables

- `--bg-0` to `--bg-3` - Background colors (lightest to darkest)
- `--text-0`, `--text-1`, `--text-dim` - Text colors
- `--accent`, `--brand` - Primary accent colors
- `--panel`, `--panel-2`, `--panel-3` - Panel/section backgrounds
- `--chip`, `--badge-bg` - Component backgrounds
- `--hair` - Subtle borders and dividers
- `--ok`, `--warn`, `--err` - Status colors
- `--glass` - Semi-transparent overlays
- `--focus` - Focus ring styles

## Theme Persistence

The theme preference is automatically saved to localStorage and restored on page load.

## System Theme Support

When set to 'system', the theme automatically follows the user's OS preference (`prefers-color-scheme: dark`).

## Transitions

Smooth transitions are automatically applied when switching between themes.