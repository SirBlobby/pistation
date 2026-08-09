<script lang="ts">
  let {
    size = 32,
    class: className = ""
  }: {
    size?: number;
    class?: string;
  } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 32 32"
  shape-rendering="crispEdges"
  class={className}
  role="img"
  aria-label="PiStation"
>
  <title>PiStation</title>

  <rect x="2" y="4" width="28" height="19" fill="#4f7cff" />
  <rect x="4" y="6" width="24" height="14" fill="#14181d" />

  <rect x="8" y="8" width="4" height="4" fill="#f4f6f8" />
  <rect x="14" y="8" width="4" height="4" fill="#f4f6f8" />
  <rect x="20" y="8" width="4" height="4" fill="#f4f6f8" />
  <rect x="8" y="14" width="4" height="4" fill="#f4f6f8" />
  <rect x="14" y="14" width="4" height="4" fill="#f4f6f8" />
  <rect x="20" y="14" width="4" height="4" fill="#4f7cff" />

  <rect x="14" y="23" width="4" height="3" fill="#4f7cff" />
  <rect x="9" y="26" width="14" height="2" fill="#4f7cff" />
</svg>
