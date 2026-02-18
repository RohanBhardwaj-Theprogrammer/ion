(function () {
  /* ── Mobile menu ───────────────────────────────── */
  var menuBtn = document.getElementById("menuBtn");
  var nav = document.getElementById("nav");

  if (menuBtn && nav) {
    menuBtn.addEventListener("click", function () {
      var open = nav.classList.toggle("open");
      menuBtn.setAttribute("aria-expanded", open);
    });

    nav.addEventListener("click", function (e) {
      if (e.target.tagName === "A") {
        nav.classList.remove("open");
        menuBtn.setAttribute("aria-expanded", "false");
      }
    });
  }

  /* ── Active-nav on hash change ─────────────────── */
  function syncNav() {
    if (!nav) return;
    var h = location.hash || "#home";
    var links = nav.querySelectorAll("a");
    for (var i = 0; i < links.length; i++) {
      links[i].classList.toggle("active", links[i].getAttribute("href") === h);
    }
  }
  window.addEventListener("hashchange", syncNav);
  syncNav();

  /* ── Footer year ───────────────────────────────── */
  var y = document.getElementById("year");
  if (y) y.textContent = new Date().getFullYear();
})();
