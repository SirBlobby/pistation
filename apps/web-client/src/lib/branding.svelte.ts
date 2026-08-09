import type { OrganizationBranding } from "@pistation/shared-types";
import { withBrandingDefaults } from "@pistation/shared-types";

import { getBranding } from "./api";

/// Shared so the layout can paint the theme and pages can read the copy, without every
/// page fetching branding for itself.
export const branding = $state<{ value: OrganizationBranding }>({
  value: withBrandingDefaults(null)
});

let hasRequested = false;

export async function ensureBranding(): Promise<void> {
  if (hasRequested) return;
  hasRequested = true;

  try {
    branding.value = withBrandingDefaults(await getBranding());
  } catch {
    // Defaults are already in place, so a server that is not up yet just means the
    // stock palette and copy.
  }
}

export function applyBranding(loaded: OrganizationBranding): void {
  branding.value = withBrandingDefaults(loaded);
}
