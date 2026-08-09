export const WIDGET_BLOCK_KINDS = ["heading", "text", "metric", "list", "image", "divider"] as const;

export type WidgetBlockKind = (typeof WIDGET_BLOCK_KINDS)[number];

export interface HeadingBlock {
  blockId: string;
  kind: "heading";
  template: string;
}

export interface TextBlock {
  blockId: string;
  kind: "text";
  template: string;
}

export interface MetricBlock {
  blockId: string;
  kind: "metric";
  labelTemplate: string;
  valueTemplate: string;
  unit: string;
}

export interface ListBlock {
  blockId: string;
  kind: "list";
  sourcePath: string;
  itemTemplate: string;
  maxItems: number;
}

export interface ImageBlock {
  blockId: string;
  kind: "image";
  urlTemplate: string;
  fit: "cover" | "contain";
}

export interface DividerBlock {
  blockId: string;
  kind: "divider";
}

export type WidgetBlock =
  | HeadingBlock
  | TextBlock
  | MetricBlock
  | ListBlock
  | ImageBlock
  | DividerBlock;

export interface WidgetDataSource {
  url: string;
  refreshSeconds: number;
  rootPath: string;
}

export interface CustomWidgetDefinition {
  definitionId: string;
  name: string;
  blocks: WidgetBlock[];
  dataSource: WidgetDataSource | null;
  accentColor: string;
  updatedAt: number;
}

const TEMPLATE_TOKEN = /\{\{\s*([^}]+?)\s*\}\}/g;

export function resolvePath(source: unknown, path: string): unknown {
  if (!path) return source;
  let current: unknown = source;
  for (const segment of path.split(".")) {
    if (current === null || current === undefined) return undefined;
    const index = Number(segment);
    if (Array.isArray(current) && Number.isInteger(index)) {
      current = current[index];
    } else if (typeof current === "object") {
      current = (current as Record<string, unknown>)[segment];
    } else {
      return undefined;
    }
  }
  return current;
}

export function renderTemplate(template: string, data: unknown): string {
  return template.replace(TEMPLATE_TOKEN, (_match, path: string) => {
    const value = resolvePath(data, path);
    if (value === null || value === undefined) return "";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  });
}

export function createEmptyDefinition(definitionId: string): CustomWidgetDefinition {
  return {
    definitionId,
    name: "Untitled widget",
    blocks: [],
    dataSource: null,
    accentColor: "#4f7cff",
    updatedAt: Date.now()
  };
}
