<script lang="ts">
  import Icon from "@iconify/svelte";
  import { Logo } from "@pistation/ui";

  import { resolveMediaUrl } from "@pistation/shared-types";

  import { apiBaseUrl } from "$lib/api";
  import { branding as brandingStore } from "$lib/branding.svelte";
  import JoinCard from "$lib/components/JoinCard.svelte";
  import { SITE_DESCRIPTION, SITE_KEYWORDS } from "$lib/meta";

  const branding = $derived(brandingStore.value);
  const isJoinOnly = $derived(branding.landingMode === "join");

  const logoUrl = $derived(branding.logoUrl ? resolveMediaUrl(apiBaseUrl, branding.logoUrl) : "");
  const title = $derived(`${branding.name} · ${branding.headline}`);

  const features = [
    {
      icon: "ph:broadcast-bold",
      title: "Share your screen",
      body: "Publish a tab, a window or your whole desktop to the big screen. Nothing to install, no cable to hunt for, and anyone in the room can take a turn."
    },
    {
      icon: "ph:pencil-simple-bold",
      title: "Annotate live",
      body: "Draw over whatever is on screen with an adjustable pen, arrows, shapes and a highlighter. Every stroke lands in the same place on the TV as on your phone."
    },
    {
      icon: "ph:video-camera-bold",
      title: "Share a camera",
      body: "Point a phone at a whiteboard, a workbench or the room itself. Front and rear cameras both work, and a screen share always takes priority."
    },
    {
      icon: "ph:hand-pointing-bold",
      title: "Point things out",
      body: "Pointer mode shows the room where you are gesturing without leaving a mark, and everyone's cursor carries their name."
    },
    {
      icon: "ph:scribble-loop-bold",
      title: "Shared whiteboard",
      body: "Flip the room into a full Excalidraw canvas that everyone can edit at once, mirrored live onto the screen."
    },
    {
      icon: "ph:squares-four-bold",
      title: "Idle dashboard",
      body: "When nobody is presenting the screen becomes a dashboard. Clock, weather, agenda, rotating wallpapers, or a widget you build yourself."
    },
    {
      icon: "ph:moon-bold",
      title: "Night mode",
      body: "Outside working hours the display drops to just the time and the join code, dimmed to whatever level suits the room."
    }
  ];

  const steps = [
    {
      title: "Plug in the Pi",
      body: "A Raspberry Pi Zero 2 W, 4 or 5 connects to any TV or monitor over HDMI and boots straight into the kiosk. One command sets it up over SSH."
    },
    {
      title: "Read the code",
      body: "The screen shows a six digit code that rotates every minute, so an old photo of it is worthless to anyone."
    },
    {
      title: "Take the screen",
      body: "Type the code here and give your name, then share a screen or a camera. You keep the screen until you leave, even as the code carries on rotating."
    }
  ];
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="keywords" content={SITE_KEYWORDS} />
  <meta property="og:title" content={title} />
  <meta name="twitter:title" content={title} />
  <meta name="description" content={SITE_DESCRIPTION} />
</svelte:head>

{#if isJoinOnly}
  <main class="flex min-h-screen flex-col items-center justify-center gap-8 px-6 py-16">
    <div class="flex flex-col items-center gap-4 text-center">
      {#if logoUrl}
        <img src={logoUrl} alt="" class="h-16 w-16 object-contain" />
      {:else}
        <Logo size={64} />
      {/if}
      <h1 class="text-3xl font-semibold tracking-tight">{branding.name}</h1>
      {#if branding.description}
        <p class="max-w-md text-ink-2">{branding.description}</p>
      {/if}
    </div>

    <JoinCard label={branding.joinLabel} />
  </main>
{:else}
<div class="flex min-h-screen flex-col">
  <header class="sticky top-0 z-10 bg-surface-0/90 backdrop-blur">
    <nav class="mx-auto flex w-full max-w-6xl items-center gap-4 px-6 py-4">
      <a href="/" class="flex items-center gap-3">
        {#if logoUrl}
          <img src={logoUrl} alt="" class="h-9 w-9 object-contain" />
        {:else}
          <Logo size={36} />
        {/if}
        <span class="text-lg font-semibold tracking-tight">{branding.name}</span>
      </a>

      <div class="ml-auto flex items-center gap-1">
        <a
          href="#how-it-works"
          class="hidden px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-1 sm:block"
        >
          How it works
        </a>
        <a
          href="#features"
          class="hidden px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-1 sm:block"
        >
          Features
        </a>
        <a
          href="#self-host"
          class="hidden px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-1 sm:block"
        >
          Self host
        </a>
        <a
          href="/admin"
          class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
        >
          <Icon icon="ph:shield-check-bold" width="16" />
          Admin
        </a>
      </div>
    </nav>
  </header>

  <main class="flex-1">
    <section class="mx-auto w-full max-w-6xl px-6 py-16 sm:py-24">
      <div class="grid items-center gap-12 lg:grid-cols-[1.1fr_1fr]">
        <div>
          <h1 class="text-4xl leading-[1.05] font-semibold tracking-tight sm:text-6xl">
            {branding.headline}
          </h1>

          <p class="mt-6 max-w-xl text-lg text-ink-1">{branding.description}</p>
        </div>

        <JoinCard label={branding.joinLabel} />
      </div>
    </section>

    <section id="how-it-works" class="bg-surface-1">
      <div class="mx-auto w-full max-w-6xl px-6 py-16 sm:py-20">
        <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">How it works</h2>
        <p class="mt-3 max-w-2xl text-2xl font-semibold tracking-tight sm:text-3xl">
          Three steps, no setup for the people joining.
        </p>

        <div class="mt-10 grid gap-px sm:grid-cols-3">
          {#each steps as step, index}
            <div class="flex flex-col gap-3 bg-surface-0 p-8">
              <span class="font-mono text-4xl font-semibold text-accent">
                {String(index + 1).padStart(2, "0")}
              </span>
              <h3 class="text-lg font-medium">{step.title}</h3>
              <p class="text-ink-2">{step.body}</p>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <section id="features" class="mx-auto w-full max-w-6xl px-6 py-16 sm:py-20">
      <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Features</h2>
      <p class="mt-3 max-w-2xl text-2xl font-semibold tracking-tight sm:text-3xl">
        Everything the room needs on one screen.
      </p>

      <div class="mt-10 grid gap-px sm:grid-cols-2 lg:grid-cols-3">
        {#each features as feature}
          <div class="flex flex-col gap-3 bg-surface-1 p-8">
            <Icon icon={feature.icon} width="26" class="text-accent" />
            <h3 class="text-lg font-medium">{feature.title}</h3>
            <p class="text-ink-2">{feature.body}</p>
          </div>
        {/each}
      </div>
    </section>

    <section id="self-host" class="bg-surface-1">
      <div class="mx-auto grid w-full max-w-6xl gap-10 px-6 py-16 sm:py-20 lg:grid-cols-2">
        <div>
          <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Self host</h2>
          <p class="mt-3 text-2xl font-semibold tracking-tight sm:text-3xl">
            Your screens, your server, your data.
          </p>
          <p class="mt-4 text-ink-1">
            PiStation ships as a Docker Compose stack: a LiveKit media server, a Rust backend and
            this site. Media travels directly between devices on your own network, so a
            presentation never touches anyone else's infrastructure.
          </p>

          <div class="mt-6 flex flex-col gap-3">
            {#each ["One command to bring the whole stack up", "SQLite for storage, no external database", "Add as many kiosks and rooms as you like", "Every kiosk reports its own CPU, memory and signal strength"] as line}
              <p class="flex items-start gap-3 text-ink-2">
                <Icon icon="ph:check-bold" width="18" class="mt-1 shrink-0 text-success" />
                {line}
              </p>
            {/each}
          </div>
        </div>

        <div class="flex flex-col justify-center bg-surface-0 p-6 sm:p-8">
          <p class="mb-3 text-xs tracking-wide text-ink-2 uppercase">Get started</p>
          <pre class="overflow-x-auto font-mono text-sm text-ink-1"><code
              >cp infra/.env.example infra/.env
docker compose -f infra/docker-compose.yml up -d</code
            ></pre>
          <p class="mt-4 text-sm text-ink-2">
            Then sign in to the admin panel, add a kiosk, and copy its enrollment token onto the Pi.
          </p>
        </div>
      </div>
    </section>
  </main>

  <footer class="mx-auto flex w-full max-w-6xl flex-wrap items-center gap-4 px-6 py-8">
    <p class="text-sm text-ink-2">
      {branding.name}{branding.footerNote ? ` · ${branding.footerNote}` : ""}
    </p>

    <div class="ml-auto flex flex-wrap items-center gap-2">
      {#each branding.links as link (link.linkId)}
        {#if link.label && link.url}
          <a
            href={link.url}
            target="_blank"
            rel="noopener noreferrer"
            class="px-4 py-2 text-sm text-ink-2 transition-colors hover:text-ink-0"
          >
            {link.label}
          </a>
        {/if}
      {/each}

      {#if branding.showSourceLink}
        <a
          href="https://github.com/SirBlobby/pistation"
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-2 bg-surface-1 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-2 hover:text-ink-0"
        >
          <Icon icon="ph:github-logo-bold" width="16" />
          Source on GitHub
        </a>
      {/if}
    </div>
  </footer>
</div>
{/if}
