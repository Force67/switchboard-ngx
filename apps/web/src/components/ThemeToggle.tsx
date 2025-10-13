import { useTheme } from "../contexts/ThemeContext";

export default function ThemeToggle() {
  const { effectiveTheme, toggleTheme } = useTheme();

  const isDarkMode = () => effectiveTheme() === 'dark';

  return (
    <button
      class="icircle"
      onClick={toggleTheme}
      title={isDarkMode() ? "Switch to light mode" : "Switch to dark mode"}
    >
      {isDarkMode() ? (
        <svg viewBox="0 0 14 14">
          <circle cx="7" cy="7" r="3" />
          <path d="M7 0v2M7 12v2M0 7h2M12 7h2M1.5 1.5l1.5 1.5M11 11l1.5 1.5M1.5 12.5l1.5-1.5M11 3l1.5-1.5" />
        </svg>
      ) : (
        <svg viewBox="0 0 14 14">
          <path d="M7 1L5 7H8L6 13L9 7H6L7 1Z" />
        </svg>
      )}
    </button>
  );
}