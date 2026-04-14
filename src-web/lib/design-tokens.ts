// Author: Quadri Atharu

export const colors = {
  primary: "#1B4332",
  primaryHover: "#2D6A4F",
  primaryLight: "rgba(27,67,50,0.12)",
  secondary: "#2D6A4F",
  accent: "#D4AF37",
  accentHover: "#C4A030",
  accentLight: "rgba(212,175,55,0.12)",
  background: "#F8F9FA",
  surface: "#FFFFFF",
  surfaceHover: "#F1F3F5",
  surfaceElevated: "#FFFFFF",
  border: "#DEE2E6",
  borderSubtle: "#E9ECEF",
  text: "#1A1A2E",
  textSecondary: "#495057",
  textTertiary: "#868E96",
  textInverse: "#FFFFFF",
  error: "#DC3545",
  errorLight: "rgba(220,53,69,0.12)",
  success: "#198754",
  successLight: "rgba(25,135,84,0.12)",
  warning: "#FFC107",
  warningLight: "rgba(255,193,7,0.12)",
  info: "#0DCAF0",
  infoLight: "rgba(13,202,240,0.12)",
} as const;

export const spacing = {
  space1: "4px",
  space2: "8px",
  space3: "12px",
  space4: "16px",
  space5: "20px",
  space6: "24px",
  space8: "32px",
  space10: "40px",
  space12: "48px",
} as const;

export const typography = {
  fontFamily: {
    ui: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    heading: '"DM Serif Display", Georgia, serif',
    mono: '"JetBrains Mono", "Fira Code", monospace',
  },
  fontSize: {
    xs: "0.75rem",
    sm: "0.875rem",
    base: "1rem",
    lg: "1.125rem",
    xl: "1.25rem",
    "2xl": "1.5rem",
    "3xl": "1.875rem",
    "4xl": "2.25rem",
  },
  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
  lineHeight: {
    xs: 1.5,
    sm: 1.5,
    base: 1.6,
    lg: 1.5,
    xl: 1.4,
    "2xl": 1.3,
    "3xl": 1.2,
    "4xl": 1.1,
  },
} as const;

export const shadows = {
  xs: "0 1px 2px rgba(0,0,0,0.04)",
  sm: "0 2px 8px rgba(0,0,0,0.08)",
  md: "0 4px 16px rgba(0,0,0,0.10)",
  lg: "0 8px 24px rgba(0,0,0,0.12)",
  xl: "0 12px 40px rgba(0,0,0,0.16)",
} as const;

export const radii = {
  sm: "4px",
  md: "8px",
  lg: "12px",
  xl: "16px",
  full: "9999px",
} as const;

export const transitions = {
  fast: "150ms ease",
  base: "200ms ease",
  slow: "300ms ease",
} as const;

export const breakpoints = {
  mobile: "375px",
  tablet: "768px",
  desktop: "1024px",
} as const;

export const layout = {
  sidebarWidth: "260px",
  sidebarMiniWidth: "72px",
  topbarHeight: "56px",
} as const;
