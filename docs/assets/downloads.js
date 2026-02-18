(function () {
  function detectOS() {
    const ua = navigator.userAgent.toLowerCase();
    const platform = (navigator.platform || "").toLowerCase();

    if (platform.includes("win") || ua.includes("windows")) return "windows";
    if (platform.includes("mac") || ua.includes("mac os")) return "macos";
    if (platform.includes("linux") || ua.includes("x11")) return "linux";
    return "unknown";
  }

  function inferRepoFromLocation() {
    const host = window.location.hostname;
    const pathParts = window.location.pathname.split("/").filter(Boolean);

    if (host.endsWith("github.io")) {
      const owner = host.replace(".github.io", "");
      const repo = pathParts.length ? pathParts[0] : null;
      if (owner && repo) return { owner, repo };
    }

    return {
      owner: "OWNER",
      repo: "REPO"
    };
  }

  async function getLatestRelease(owner, repo) {
    const url = `https://api.github.com/repos/${owner}/${repo}/releases/latest`;
    const response = await fetch(url, {
      headers: { Accept: "application/vnd.github+json" }
    });
    if (!response.ok) {
      throw new Error("Unable to fetch release metadata");
    }
    return response.json();
  }

  function chooseAsset(assets, os) {
    const list = Array.isArray(assets) ? assets : [];
    const needle = os === "windows" ? "windows" : os === "macos" ? "macos" : os === "linux" ? "linux" : "";
    if (!needle) return null;

    return list.find((asset) => {
      const name = String(asset.name || "").toLowerCase();
      return name.includes("ion") && name.includes(needle) && name.includes("x86_64");
    }) || null;
  }

  function fallbackLinks(owner, repo) {
    const base = `https://github.com/${owner}/${repo}/releases/latest/download`;
    return {
      windows: `${base}/ion-windows-x86_64.exe`,
      linux: `${base}/ion-linux-x86_64`,
      releasePage: `https://github.com/${owner}/${repo}/releases/latest`
    };
  }

  async function setupDownloadUI() {
    const primary = document.getElementById("primaryDownload");
    if (!primary) return;

    const hint = document.getElementById("downloadHint");
    const detectedOS = document.getElementById("detectedOS");
    const latestLink = document.getElementById("latestReleaseLink");
    const winLink = document.getElementById("winLink");
    const linuxLink = document.getElementById("linuxLink");

    const os = detectOS();
    const { owner, repo } = inferRepoFromLocation();
    const links = fallbackLinks(owner, repo);

    if (latestLink) latestLink.href = links.releasePage;
    if (winLink) winLink.href = links.windows;
    if (linuxLink) linuxLink.href = links.linux;

    const osLabel = os === "windows" ? "Windows" : os === "macos" ? "macOS" : os === "linux" ? "Linux" : "Unknown";
    primary.textContent = `Download for ${osLabel}`;
    if (detectedOS) detectedOS.textContent = `Detected: ${osLabel}`;

    // fallback first (works even if API rate-limited)
    primary.href = os === "windows" ? links.windows : os === "linux" ? links.linux : links.releasePage;

    if (os === "macos") {
      if (hint) hint.textContent = "macOS binary is not currently published. Opening latest release page instead.";
      return;
    }

    if (owner === "OWNER" || repo === "REPO") {
      if (hint) hint.textContent = "Repository auto-detection is unavailable outside GitHub Pages path. Update owner/repo in downloads.js if needed.";
      return;
    }

    try {
      const release = await getLatestRelease(owner, repo);
      const picked = chooseAsset(release.assets, os);
      if (picked && picked.browser_download_url) {
        primary.href = picked.browser_download_url;
        if (hint) hint.textContent = `Auto-selected binary: ${picked.name}`;
      } else if (hint) {
        hint.textContent = "No exact asset match found; using fallback latest link.";
      }
    } catch (error) {
      if (hint) hint.textContent = "Could not query latest release metadata; using fallback links.";
    }
  }

  setupDownloadUI();
})();
