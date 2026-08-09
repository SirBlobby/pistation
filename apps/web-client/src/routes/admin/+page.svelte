<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Kiosk } from "@pistation/shared-types";
  import { onMount } from "svelte";

  import {
    adminLogin,
    ApiError,
    createKiosk,
    deleteKiosk,
    listKiosks,
    updateKiosk,
    uploadKioskPackage
  } from "$lib/api";
  import { Logo } from "@pistation/ui";

  import InstallCommand from "$lib/components/admin/InstallCommand.svelte";
  import KioskStats from "$lib/components/admin/KioskStats.svelte";
  import { clearAdmin, loadAdmin, saveAdmin, type StoredAdmin } from "$lib/session";

  let admin = $state<StoredAdmin | null>(null);
  let kiosks = $state<Kiosk[]>([]);
  let errorMessage = $state<string | null>(null);
  let isBusy = $state(false);

  let email = $state("");
  let password = $state("");

  let newName = $state("");
  let newLocation = $state("");
  let issuedToken = $state<{ name: string; token: string } | null>(null);
  let editingId = $state<string | null>(null);
  let editName = $state("");
  let editLocation = $state("");

  function startRename(kiosk: Kiosk) {
    editingId = kiosk.kioskId;
    editName = kiosk.name;
    editLocation = kiosk.location;
  }

  function cancelRename() {
    editingId = null;
  }

  async function saveRename(event: SubmitEvent) {
    event.preventDefault();
    if (!admin || !editingId || !editName.trim()) return;

    isBusy = true;
    errorMessage = null;

    try {
      await updateKiosk(admin.accessToken, editingId, editName, editLocation);
      editingId = null;
      await refresh();
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not rename the kiosk.";
    } finally {
      isBusy = false;
    }
  }

  let packageInput = $state<HTMLInputElement | null>(null);
  let isUploadingPackage = $state(false);
  let packageMessage = $state<string | null>(null);

  async function handlePackageUpload(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file || !admin) return;

    isUploadingPackage = true;
    packageMessage = null;
    errorMessage = null;

    try {
      const result = await uploadKioskPackage(admin.accessToken, file);
      const megabytes = (result.sizeBytes / 1024 / 1024).toFixed(1);
      packageMessage = `Uploaded ${file.name}, ${megabytes} MB. Kiosks will install this build.`;
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not upload the package.";
    } finally {
      isUploadingPackage = false;
    }
  }

  onMount(() => {
    admin = loadAdmin();
    if (admin) void refresh();

    const interval = setInterval(() => {
      if (admin && !editingId) void refresh();
    }, 15000);

    return () => clearInterval(interval);
  });

  async function signIn(event: SubmitEvent) {
    event.preventDefault();
    isBusy = true;
    errorMessage = null;

    try {
      const response = await adminLogin(email, password);
      const stored = {
        accessToken: response.accessToken,
        email: response.email,
        expiresAt: response.expiresAt
      };
      saveAdmin(stored);
      admin = stored;
      password = "";
      await refresh();
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Sign in failed.";
    } finally {
      isBusy = false;
    }
  }

  function signOut() {
    clearAdmin();
    admin = null;
    kiosks = [];
  }

  async function refresh() {
    if (!admin) return;
    try {
      const response = await listKiosks(admin.accessToken);
      kiosks = response.kiosks;
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) signOut();
      else errorMessage = "Could not load kiosks.";
    }
  }

  async function addKiosk(event: SubmitEvent) {
    event.preventDefault();
    if (!admin || !newName.trim()) return;

    isBusy = true;
    try {
      const response = await createKiosk(admin.accessToken, newName, newLocation);
      issuedToken = { name: response.kiosk.name, token: response.enrollmentToken };
      newName = "";
      newLocation = "";
      await refresh();
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not create kiosk.";
    } finally {
      isBusy = false;
    }
  }

  async function removeKiosk(kiosk: Kiosk) {
    if (!admin) return;
    if (!confirm(`Delete ${kiosk.name}? This cannot be undone.`)) return;

    await deleteKiosk(admin.accessToken, kiosk.kioskId);
    await refresh();
  }

  function formatLastSeen(timestamp: number | null) {
    if (!timestamp) return "never";
    return new Date(timestamp).toLocaleString();
  }
</script>

<svelte:head>
  <title>Admin · PiStation</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

{#if !admin}
  <main class="flex min-h-screen flex-col items-center justify-center px-6 py-12">
    <div class="w-full max-w-sm">
      <a
        href="/"
        class="mb-8 inline-flex items-center gap-2 text-sm text-ink-2 transition-colors hover:text-ink-0"
      >
        <Icon icon="ph:arrow-left-bold" width="16" />
        Back to PiStation
      </a>

      <div class="mb-8 flex flex-col items-center text-center">
        <Logo size={56} class="mb-4" />
        <h1 class="text-2xl font-semibold tracking-tight">PiStation admin</h1>
        <p class="mt-1 text-sm text-ink-2">Sign in to manage kiosks</p>
      </div>

      {#if errorMessage}
        <p class="mb-6 flex items-center gap-2 bg-danger/15 px-4 py-3 text-sm text-danger">
          <Icon icon="ph:warning-circle-bold" width="18" class="shrink-0" />
          {errorMessage}
        </p>
      {/if}

      <form onsubmit={signIn} class="bg-surface-1 p-6">
        <label class="mb-4 block">
          <span class="mb-2 block text-sm font-medium text-ink-1">Email</span>
          <input
            bind:value={email}
            type="email"
            required
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          />
        </label>

        <label class="mb-6 block">
          <span class="mb-2 block text-sm font-medium text-ink-1">Password</span>
          <input
            bind:value={password}
            type="password"
            required
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          />
        </label>

        <button
          type="submit"
          disabled={isBusy}
          class="w-full bg-accent px-6 py-3 font-semibold text-white transition-colors hover:bg-accent-strong disabled:bg-surface-3"
        >
          Sign in
        </button>
      </form>
    </div>
  </main>
{:else}
  <main class="mx-auto w-full max-w-5xl px-6 py-12">
    <header class="mb-10 flex flex-wrap items-center gap-3">
      <a
        href="/"
        aria-label="Back to PiStation"
        class="flex h-10 w-10 items-center justify-center bg-surface-2 text-ink-1 transition-colors hover:bg-surface-3"
      >
        <Icon icon="ph:arrow-left-bold" width="18" />
      </a>

      <Logo size={40} />

      <div class="flex-1">
        <h1 class="text-xl font-semibold tracking-tight">PiStation admin</h1>
        <p class="text-sm text-ink-2">{admin.email}</p>
      </div>

      <a
        href="/admin/organization"
        class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
      >
        <Icon icon="ph:buildings-bold" width="16" />
        Organisation
      </a>

      <button
        onclick={signOut}
        class="bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
      >
        Sign out
      </button>
    </header>

    {#if errorMessage}
      <p class="mb-6 flex items-center gap-2 bg-danger/15 px-4 py-3 text-sm text-danger">
        <Icon icon="ph:warning-circle-bold" width="18" class="shrink-0" />
        {errorMessage}
      </p>
    {/if}

    {#if issuedToken}
      <div class="mb-8 flex flex-col gap-3">
        <InstallCommand enrollmentToken={issuedToken.token} kioskName={issuedToken.name} />
        <button
          onclick={() => (issuedToken = null)}
          class="self-start bg-surface-2 px-4 py-2 text-sm text-ink-1 hover:bg-surface-3"
        >
          Done
        </button>
      </div>
    {/if}

    <section class="mb-10 flex flex-col gap-3 bg-surface-1 p-5">
      <div class="flex flex-wrap items-center gap-3">
        <Icon icon="ph:package-bold" width="18" class="text-accent" />
        <h2 class="flex-1 text-sm font-semibold tracking-wide uppercase">Kiosk build</h2>

        <input
          bind:this={packageInput}
          type="file"
          accept=".deb"
          onchange={handlePackageUpload}
          class="hidden"
        />

        <button
          onclick={() => packageInput?.click()}
          disabled={isUploadingPackage}
          class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3 disabled:text-ink-2"
        >
          {#if isUploadingPackage}
            <Icon icon="ph:circle-notch-bold" width="16" class="animate-spin" />
            Uploading
          {:else}
            <Icon icon="ph:upload-simple-bold" width="16" />
            Upload .deb
          {/if}
        </button>
      </div>

      <p class="text-sm text-ink-2">
        The installer script downloads this package onto each Pi. Upload the arm64 build produced
        by the release workflow, then every kiosk is one command to set up.
      </p>

      {#if packageMessage}
        <p class="bg-success/15 px-4 py-2 text-sm text-success">{packageMessage}</p>
      {/if}
    </section>

    <section class="mb-10">
      <h2 class="mb-4 text-sm font-semibold tracking-wide text-ink-2 uppercase">Kiosks</h2>

      {#if kiosks.length === 0}
        <p class="bg-surface-1 px-6 py-8 text-center text-ink-2">
          No kiosks yet. Add one below to get an enrollment token.
        </p>
      {/if}

      <div class="flex flex-col gap-px">
        {#each kiosks as kiosk (kiosk.kioskId)}
          <div class="flex flex-wrap items-center gap-4 bg-surface-1 px-5 py-4">
            <span
              class="h-2 w-2 shrink-0"
              class:bg-success={kiosk.status === "online"}
              class:bg-ink-2={kiosk.status !== "online"}
            ></span>

            {#if editingId === kiosk.kioskId}
              <form onsubmit={saveRename} class="flex min-w-60 flex-1 flex-wrap items-center gap-2">
                <input
                  bind:value={editName}
                  placeholder="Name"
                  required
                  class="min-w-32 flex-1 bg-surface-2 px-3 py-2 text-sm text-ink-0"
                />
                <input
                  bind:value={editLocation}
                  placeholder="Location"
                  class="min-w-32 flex-1 bg-surface-2 px-3 py-2 text-sm text-ink-0"
                />
                <button
                  type="submit"
                  disabled={isBusy || !editName.trim()}
                  class="flex items-center gap-2 bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-strong disabled:bg-surface-3 disabled:text-ink-2"
                >
                  <Icon icon="ph:check-bold" width="16" />
                  Save
                </button>
                <button
                  type="button"
                  onclick={cancelRename}
                  class="bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
                >
                  Cancel
                </button>
              </form>
            {:else}
              <div class="min-w-40 flex-1">
                <p class="font-medium">{kiosk.name}</p>
                <p class="text-sm text-ink-2">
                  {kiosk.location || "No location"} · last seen {formatLastSeen(kiosk.lastSeenAt)}
                </p>
                {#if kiosk.status === "online" && kiosk.metrics}
                  <div class="mt-1.5">
                    <KioskStats metrics={kiosk.metrics} metricsAt={kiosk.metricsAt} compact />
                  </div>
                {/if}
              </div>

              <button
                onclick={() => startRename(kiosk)}
                aria-label={`Rename ${kiosk.name}`}
                class="flex h-9 w-9 items-center justify-center bg-surface-2 text-ink-1 transition-colors hover:bg-surface-3"
              >
                <Icon icon="ph:pencil-simple-bold" width="16" />
              </button>

              <a
                href={`/admin/kiosks/${kiosk.kioskId}`}
                class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
              >
                <Icon icon="ph:squares-four-bold" width="16" />
                Widgets
              </a>

              <button
                onclick={() => removeKiosk(kiosk)}
                aria-label={`Delete ${kiosk.name}`}
                class="flex h-9 w-9 items-center justify-center bg-surface-2 text-danger transition-colors hover:bg-surface-3"
              >
                <Icon icon="ph:trash-bold" width="16" />
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </section>

    <section class="bg-surface-1 p-6">
      <h2 class="mb-4 text-sm font-semibold tracking-wide text-ink-2 uppercase">Add a kiosk</h2>
      <form onsubmit={addKiosk} class="flex flex-wrap gap-3">
        <input
          bind:value={newName}
          placeholder="Name"
          required
          class="min-w-40 flex-1 bg-surface-2 px-4 py-3 text-ink-0 placeholder:text-ink-2"
        />
        <input
          bind:value={newLocation}
          placeholder="Location"
          class="min-w-40 flex-1 bg-surface-2 px-4 py-3 text-ink-0 placeholder:text-ink-2"
        />
        <button
          type="submit"
          disabled={isBusy}
          class="flex items-center gap-2 bg-accent px-6 py-3 font-semibold text-white transition-colors hover:bg-accent-strong disabled:bg-surface-3"
        >
          <Icon icon="ph:plus-bold" width="18" />
          Create
        </button>
      </form>
    </section>
  </main>
{/if}
