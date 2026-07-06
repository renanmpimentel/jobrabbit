// Brand badge — keep in sync with assets/brand/logo.svg.
// Rabbit glyph based on the Lucide "rabbit" icon (ISC license).
export default function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 32" className={className} aria-hidden="true">
      <rect width="32" height="32" rx="7.5" fill="#4F46E5" />
      <g
        fill="none"
        stroke="#FFFFFF"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        transform="translate(8,5.6) scale(0.78)"
      >
        <path d="M13 16a3 3 0 0 1 2.24 5" />
        <path d="M18 12h.01" />
        <path d="M18 21h-8a4 4 0 0 1-4-4 7 7 0 0 1 7-7h.2L9.6 6.4a1 1 0 1 1 2.8-2.8L15.8 7h.2c3.3 0 6 2.7 6 6v1a2 2 0 0 1-2 2h-1a3 3 0 0 0-3 3" />
        <path d="M20 8.54V4a2 2 0 1 0-4 0v3" />
        <path d="M7.612 12.524a3 3 0 1 0-1.6 4.3" />
      </g>
      <g stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" opacity="0.75">
        <path d="M4.5 10.5h3" />
        <path d="M3.5 15.5h2.6" />
        <path d="M5 20.5h2" />
      </g>
    </svg>
  );
}
