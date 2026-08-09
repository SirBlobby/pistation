import type { HandleServerError } from "@sveltejs/kit";

export const handleError: HandleServerError = ({ error, event }) => {
  const detail = error instanceof Error ? (error.stack ?? error.message) : String(error);
  console.error(`[error] ${event.request.method} ${event.url.pathname}\n${detail}`);

  return {
    message: "Something went wrong rendering this page."
  };
};
