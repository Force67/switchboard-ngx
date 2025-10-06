export default function SidebarSearch() {
  return (
    <div class="srch">
      <svg viewBox="0 0 14 14">
        <circle cx="5.5" cy="5.5" r="4.5" stroke-width="1.5" fill="none" />
        <path d="M9.5 9.5L12.5 12.5" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <input type="text" placeholder="Search your threads…" />
    </div>
  );
}