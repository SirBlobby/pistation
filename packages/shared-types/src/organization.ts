export interface BrandLink {
  linkId: string;
  label: string;
  url: string;
}

export const LANDING_MODES = ["full", "join"] as const;

export type LandingMode = (typeof LANDING_MODES)[number];

export interface BrandTheme {
  surface0: string;
  surface1: string;
  surface2: string;
  surface3: string;
  ink0: string;
  ink1: string;
  ink2: string;
}

export const DEFAULT_THEME: BrandTheme = {
  surface0: "#0b0d10",
  surface1: "#14181d",
  surface2: "#1c2229",
  surface3: "#262e37",
  ink0: "#f4f6f8",
  ink1: "#a8b3c0",
  ink2: "#6b7885"
};

export interface OrganizationBranding {
  name: string;
  headline: string;
  description: string;
  logoUrl: string;
  accentColor: string;
  joinLabel: string;
  footerNote: string;
  showSourceLink: boolean;
  landingMode: LandingMode;
  theme: BrandTheme;
  links: BrandLink[];
  updatedAt: number;
}

/// Inline style declaring the palette as custom properties, so every Tailwind utility
/// built on these tokens picks up the organisation's colours without touching components.
export function themeStyle(branding: OrganizationBranding): string {
  const theme = { ...DEFAULT_THEME, ...(branding.theme ?? {}) };

  return [
    `--color-surface-0: ${theme.surface0}`,
    `--color-surface-1: ${theme.surface1}`,
    `--color-surface-2: ${theme.surface2}`,
    `--color-surface-3: ${theme.surface3}`,
    `--color-ink-0: ${theme.ink0}`,
    `--color-ink-1: ${theme.ink1}`,
    `--color-ink-2: ${theme.ink2}`,
    `--color-accent: ${branding.accentColor}`,
    `--color-accent-strong: ${branding.accentColor}`,
    `background-color: ${theme.surface0}`,
    `color: ${theme.ink0}`
  ].join("; ");
}

export const DEFAULT_BRANDING: OrganizationBranding = {
  name: "PiStation",
  headline: "Any screen becomes a shared screen.",
  description:
    "Type the code shown on screen to present, draw on what is being shown, or open a " +
    "whiteboard together. No accounts, no installs, and nothing leaves the network it runs on.",
  logoUrl: "",
  accentColor: "#4f7cff",
  joinLabel: "Enter the code on screen",
  footerNote: "",
  showSourceLink: true,
  landingMode: "full",
  theme: DEFAULT_THEME,
  links: [],
  updatedAt: 0
};

export function withBrandingDefaults(
  branding: Partial<OrganizationBranding> | null | undefined
): OrganizationBranding {
  if (!branding) return { ...DEFAULT_BRANDING, theme: { ...DEFAULT_THEME } };

  return {
    ...DEFAULT_BRANDING,
    ...branding,
    theme: { ...DEFAULT_THEME, ...(branding.theme ?? {}) },
    links: branding.links ?? []
  };
}
