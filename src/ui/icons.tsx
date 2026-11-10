// Unified SVG icons (stroke: currentColor, inherits text color)
type P = { size?: number; className?: string }

function svgProps(size: number, className?: string) {
  return {
    width: size,
    height: size,
    viewBox: '0 0 24 24',
    fill: 'none',
    stroke: 'currentColor',
    strokeWidth: 2,
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
    className,
    'aria-hidden': true,
  }
}

export const MonitorIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <rect x="2" y="4" width="20" height="13" rx="2" />
    <path d="M8 21h8M12 17v4" />
  </svg>
)

export const CropIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M6 2v14a2 2 0 0 0 2 2h14" />
    <path d="M18 22V8a2 2 0 0 0-2-2H2" />
  </svg>
)

export const MicIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <rect x="9" y="2" width="6" height="12" rx="3" />
    <path d="M5 10a7 7 0 0 0 14 0M12 19v3" />
  </svg>
)

export const SpeakerIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M11 5 6 9H2v6h4l5 4V5z" />
    <path d="M15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13" />
  </svg>
)

export const FolderIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
  </svg>
)

export const ChevronDownIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m6 9 6 6 6-6" /></svg>
)

export const ChevronRightIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m9 6 6 6-6 6" /></svg>
)

export const ChevronLeftIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m15 6-6 6 6 6" /></svg>
)

export const PlayIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)} fill="currentColor" stroke="none">
    <path d="M7 4.5v15l13-7.5-13-7.5z" />
  </svg>
)

export const StopIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)} fill="currentColor" stroke="none">
    <rect x="6" y="6" width="12" height="12" rx="2" />
  </svg>
)

export const TrashIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M3 6h18M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2M6 6l1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14" />
  </svg>
)

export const PlusIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="M12 5v14M5 12h14" /></svg>
)

export const ArrowUpIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="M12 19V5M5 12l7-7 7 7" /></svg>
)

export const ArrowDownIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="M12 5v14M19 12l-7 7-7-7" /></svg>
)

export const XIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="M18 6 6 18M6 6l12 12" /></svg>
)

export const CameraIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <rect x="2" y="6" width="14" height="12" rx="2" />
    <path d="m16 10 6-3v10l-6-3" />
  </svg>
)

export const MouseIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <rect x="6" y="2" width="12" height="20" rx="6" />
    <path d="M12 6v4" />
  </svg>
)

export const DownloadIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M12 3v12M6 9l6 6 6-6M4 21h16" />
  </svg>
)

export const MinusIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="M5 12h14" /></svg>
)

export const MaxIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><rect x="5" y="5" width="14" height="14" rx="1" /></svg>
)

export const PauseIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)} fill="currentColor" stroke="none">
    <rect x="6" y="5" width="4" height="14" rx="1" />
    <rect x="14" y="5" width="4" height="14" rx="1" />
  </svg>
)

export const FlagIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M5 21V4M5 4h12l-2.5 4L17 12H5" />
  </svg>
)

export const RewindToPointIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M3 12a9 9 0 1 0 3-6.7M3 4v5h5" />
    <circle cx="12" cy="12" r="1.6" fill="currentColor" stroke="none" />
  </svg>
)

export const CheckIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m4 12 5 5L20 7" /></svg>
)

export const RefreshIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <path d="M21 12a9 9 0 1 1-2.6-6.3M21 3v6h-6" />
  </svg>
)

export const RewindIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)} fill="currentColor" stroke="none">
    <path d="M11 12 20 6v12l-9-6zM3 6h2v12H3z" />
  </svg>
)

export const DoubleLeftIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m11 7-5 5 5 5M18 7l-5 5 5 5" /></svg>
)

export const DoubleRightIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}><path d="m13 7 5 5-5 5M6 7l5 5-5 5" /></svg>
)

export const LockIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <rect x="4" y="11" width="16" height="10" rx="2" />
    <path d="M8 11V7a4 4 0 0 1 8 0v4" />
  </svg>
)

export const ListIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <line x1="8" y1="6" x2="21" y2="6" />
    <line x1="8" y1="12" x2="21" y2="12" />
    <line x1="8" y1="18" x2="21" y2="18" />
    <line x1="3" y1="6" x2="3.01" y2="6" />
    <line x1="3" y1="12" x2="3.01" y2="12" />
    <line x1="3" y1="18" x2="3.01" y2="18" />
  </svg>
)

export const GearIcon = ({ size = 16, className }: P) => (
  <svg {...svgProps(size, className)}>
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1.55-1 1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h.01a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v.01a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.55 1z" />
  </svg>
)
