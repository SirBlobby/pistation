import { env } from "$env/dynamic/public";

export const apiBaseUrl = (env.PUBLIC_API_URL ?? "http://localhost:8080").replace(/\/$/, "");
