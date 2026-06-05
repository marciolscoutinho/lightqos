(function () {
  "use strict";

  var SITE = window.LIGHTQOS_SITE || {};
  var $ = function (selector) { return document.querySelector(selector); };
  var $$ = function (selector) { return Array.prototype.slice.call(document.querySelectorAll(selector)); };

  var currentLang = localStorage.getItem("lightqos-lang") || "pt";
  if (!SITE[currentLang]) currentLang = "pt";

  var THEME_KEY = "lightqos-theme";

  function getStoredTheme() {
    try {
      var stored = localStorage.getItem(THEME_KEY);
      return stored === "light" || stored === "dark" ? stored : null;
    } catch (error) {
      return null;
    }
  }

  function setTheme(themeName) {
    var safeTheme = themeName === "light" ? "light" : "dark";
    document.documentElement.dataset.theme = safeTheme;

    try {
      localStorage.setItem(THEME_KEY, safeTheme);
    } catch (error) {
      // localStorage may be unavailable in strict/private contexts.
    }

    var button = document.getElementById("theme-toggle");
    if (button) {
      button.setAttribute("aria-pressed", safeTheme === "light" ? "true" : "false");
      button.setAttribute("title", safeTheme === "light" ? "Switch to dark theme" : "Switch to light theme");
      button.textContent = safeTheme === "light" ? "☾" : "◐";
    }
  }

  function initTheme() {
    setTheme(getStoredTheme() || document.documentElement.dataset.theme || "dark");

    var theme = document.getElementById("theme-toggle");
    if (!theme || theme.dataset.bound === "true") return;

    theme.dataset.bound = "true";
    theme.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();

      var current = document.documentElement.dataset.theme === "light" ? "light" : "dark";
      setTheme(current === "light" ? "dark" : "light");
    });
  }

  function data() {
    return SITE[currentLang] || SITE.pt || {};
  }

  function setText(id, value) {
    var el = document.getElementById(id);
    if (el) el.textContent = value || "";
  }

  function renderLanguageMenu() {
    var button = $("#language-button");
    var menu = $("#language-menu");
    if (!button || !menu || !SITE.languages) return;

    var meta = SITE.languages.find(function (l) { return l.code === currentLang; }) || SITE.languages[0];
    button.innerHTML = "<span>" + meta.flag + "</span> <strong>" + meta.short + "</strong>";

    if (data().ui && data().ui.language) {
      button.setAttribute("aria-label", data().ui.language);
    }

    menu.innerHTML = SITE.languages.map(function (l) {
      return '<button type="button" data-lang="' + l.code + '"><span>' + l.flag + " " + l.label + '</span><strong>' + l.short + '</strong></button>';
    }).join("");

    menu.querySelectorAll("button").forEach(function (btn) {
      btn.addEventListener("click", function () {
        currentLang = btn.getAttribute("data-lang");
        localStorage.setItem("lightqos-lang", currentLang);
        menu.classList.remove("open");
        render();
      });
    });
  }

  function renderNav() {
    var nav = $("#nav");
    if (!nav) return;

    var n = data().nav || {};
    var currentPage = location.pathname.split("/").pop() || "index.html";
    var items = [
      { href: "index.html#visao", label: n.overview || "Vision" },
      { href: "founder.html", label: n.founder || "Founder" },
      { href: "index.html#arquitetura", label: n.architecture || "Architecture" },
      { href: "index.html#modulos", label: n.modules || "Modules" },
      { href: "index.html#tecnologia", label: n.technology || "Technology" },
      { href: "index.html#exemplos", label: n.examples || "Examples" },
      { href: "index.html#instalacao", label: n.install || "Install" }
    ];

    nav.innerHTML = items.map(function (item) {
      var active = currentPage === "founder.html" && item.href === "founder.html" ? ' class="active"' : "";
      return "<a" + active + ' href="' + item.href + '">' + item.label + "</a>";
    }).join("");
  }

  function renderHero() {
    var ui = data().ui || {};
    var hero = data().hero || {};

    setText("enter-btn", ui.enter);
    setText("hero-eyebrow", hero.eyebrow);
    setText("hero-title", hero.title);
    setText("hero-subtitle", hero.subtitle);
    setText("hero-text", hero.text);
    setText("hero-primary", hero.primary);
    setText("hero-secondary", hero.secondary);
    setText("badge-1", hero.badge1);
    setText("badge-2", hero.badge2);
    setText("badge-3", hero.badge3);
    setText("badge-4", hero.badge4);

    var repo = $("#repo-link");
    if (repo) {
      repo.href = SITE.meta && SITE.meta.github ? SITE.meta.github : "#";
      repo.textContent = ui.repo || "GitHub";
    }

    var theme = $("#theme-toggle");
    if (theme && ui.theme) theme.setAttribute("aria-label", ui.theme);
  }

  function renderStats() {
    var target = $("#stats");
    if (!target || !SITE.stats) return;

    var labels = data().statsLabels || {};
    var stats = [
      [SITE.stats.files, labels.files || "files"],
      [SITE.stats.rust, labels.rust || "Rust"],
      [SITE.stats.python, labels.python || "Python"],
      [SITE.stats.docs, labels.docs || "docs"]
    ];

    target.innerHTML = stats.map(function (item) {
      return '<article class="stat reveal"><strong>' + item[0] + "</strong><span>" + item[1] + "</span></article>";
    }).join("");
  }

  function renderContent() {
    var d = data();
    var vision = d.vision || {};
    var architecture = d.architecture || {};
    var ui = d.ui || {};

    setText("vision-title", vision.title);
    setText("vision-text", vision.text);
    setText("mission-title", vision.missionTitle);
    setText("mission-text", vision.mission);

    setText("architecture-title", architecture.title);
    setText("architecture-subtitle", architecture.subtitle);

    setText("modules-title", d.modulesTitle);
    setText("modules-subtitle", d.modulesSubtitle);
    setText("tech-title", d.techTitle);
    setText("examples-title", d.examplesTitle);
    setText("examples-subtitle", d.examplesSubtitle);
    setText("install-title", d.installTitle);
    setText("terminal-title", ui.terminalTitle);
    setText("footer", d.footer);
  }

  function renderArchitectureCards() {
    var target = $("#architecture-cards");
    if (!target) return;

    var d = data();
    var sub = $("#architecture-cards-head");
    if (sub) {
      sub.innerHTML = "<h3>" + (d.architectureCardsTitle || "") + "</h3><p>" + (d.architectureCardsSubtitle || "") + "</p>";
    }

    var cards = d.architectureCards || [];
    target.innerHTML = cards.map(function (card) {
      return '<article class="arch-card reveal">' +
        '<img src="' + card.img + '" alt="' + card.title + '">' +
        '<div class="arch-card-body"><h3>' + card.title + '</h3><p>' + card.desc + '</p></div>' +
        '</article>';
    }).join("");
  }

  function renderLayers() {
    var target = $("#layers-grid");
    if (!target) return;

    target.innerHTML = (data().layers || []).map(function (layer) {
      return '<article class="card reveal"><span class="badge">' + layer.name + '</span><h3>' + layer.full + '</h3><p>' + layer.desc + '</p></article>';
    }).join("");
  }

  function renderModules() {
    var target = $("#modules-grid");
    if (!target) return;

    target.innerHTML = (data().modules || []).map(function (m) {
      return '<article class="card reveal"><span class="badge">' + m.status + '</span><h3>' + m.name + '</h3><p>' + m.desc + '</p></article>';
    }).join("");
  }

  function renderTechExamplesInstall() {
    var tech = $("#tech-cloud");
    if (tech) {
      tech.innerHTML = (data().tech || []).map(function (t) { return "<span>" + t + "</span>"; }).join("");
    }

    var examples = $("#examples-list");
    if (examples) {
      examples.innerHTML = (data().examples || []).map(function (e) { return "<code>" + e + "</code>"; }).join("");
    }

    var commands = $("#commands-output");
    if (commands) {
      commands.textContent = (data().commands || []).map(function (c) { return "$ " + c; }).join("\n");
    }
  }

  function renderFounderPage() {
    var title = $("#founder-title");
    if (!title) return;

    var founder = data().founder || {};
    setText("founder-title", founder.title);
    setText("founder-subtitle", founder.subtitle);
    setText("founder-bio1", founder.bio1);
    setText("founder-bio2", founder.bio2);

    var cta = $("#founder-cta");
    if (cta) {
      cta.textContent = founder.cta || "GitHub";
      cta.href = SITE.meta && SITE.meta.github ? SITE.meta.github : "#";
    }
  }

  function revealOnScroll() {
    var elements = $$(".reveal");
    if (!("IntersectionObserver" in window)) {
      elements.forEach(function (el) { el.classList.add("visible"); });
      return;
    }

    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) entry.target.classList.add("visible");
      });
    }, { threshold: 0.12 });

    elements.forEach(function (el) { observer.observe(el); });
  }

  function bootSequence() {
    var boot = $("#boot");
    var bootLines = $("#boot-lines");
    if (!boot || !bootLines) return;

    bootLines.innerHTML = "";
    var lines = data().ui && data().ui.boot ? data().ui.boot : ["LightQOS ready"];
    var i = 0;

    function add() {
      if (i >= lines.length) {
        setTimeout(function () { boot.classList.add("hidden"); }, 650);
        return;
      }

      var p = document.createElement("p");
      p.innerHTML = "<strong>[OK]</strong> " + lines[i];
      bootLines.appendChild(p);
      i += 1;
      setTimeout(add, 300);
    }

    add();
  }

  function initCursor() {
    var dot = $("#cursor-dot");
    var ring = $("#cursor-ring");
    if (!dot || !ring) return;

    var isTouch = window.matchMedia && window.matchMedia("(hover: none)").matches;
    if (isTouch) {
      dot.style.display = "none";
      ring.style.display = "none";
      return;
    }

    var mx = 0, my = 0, rx = 0, ry = 0;

    window.addEventListener("mousemove", function (event) {
      mx = event.clientX;
      my = event.clientY;
      dot.style.left = mx + "px";
      dot.style.top = my + "px";
    });

    function animate() {
      rx += (mx - rx) * 0.16;
      ry += (my - ry) * 0.16;
      ring.style.left = rx + "px";
      ring.style.top = ry + "px";
      requestAnimationFrame(animate);
    }
    animate();

    document.addEventListener("mouseover", function (event) {
      if (event.target.closest("a, button")) document.body.classList.add("cursor-active");
    });

    document.addEventListener("mouseout", function (event) {
      if (event.target.closest("a, button")) document.body.classList.remove("cursor-active");
    });
  }


  function initAnchorOffset() {
    document.addEventListener("click", function (event) {
      var link = event.target.closest('a[href^="#"], a[href^="index.html#"]');
      if (!link) return;

      var href = link.getAttribute("href");
      var hash = href.indexOf("#") >= 0 ? href.slice(href.indexOf("#")) : href;
      if (!hash || hash === "#") return;

      var currentPage = location.pathname.split("/").pop() || "index.html";
      if (href.indexOf("index.html#") === 0 && currentPage !== "index.html" && currentPage !== "") {
        return;
      }

      var target = document.querySelector(hash);
      if (!target) return;

      event.preventDefault();

      var header = document.querySelector(".site-header");
      var headerHeight = header ? header.offsetHeight : 100;
      var extraGap = 14;
      var top = target.getBoundingClientRect().top + window.pageYOffset - headerHeight - extraGap;

      window.history.pushState(null, "", hash);
      window.scrollTo({
        top: Math.max(0, top),
        behavior: "smooth"
      });
    });

    if (location.hash) {
      setTimeout(function () {
        var target = document.querySelector(location.hash);
        if (!target) return;

        var header = document.querySelector(".site-header");
        var headerHeight = header ? header.offsetHeight : 100;
        var extraGap = 14;
        var top = target.getBoundingClientRect().top + window.pageYOffset - headerHeight - extraGap;

        window.scrollTo({
          top: Math.max(0, top),
          behavior: "auto"
        });
      }, 180);
    }
  }



  function initArchitectureDragScroll() {
    var slider = document.querySelector(".architecture-cards");
    if (!slider) return;

    var isDown = false;
    var startX = 0;
    var scrollLeft = 0;
    var moved = false;

    function stopDrag() {
      isDown = false;
      slider.classList.remove("dragging");
    }

    slider.addEventListener("mousedown", function (e) {
      if (window.innerWidth <= 680) return;
      isDown = true;
      moved = false;
      slider.classList.add("dragging");
      startX = e.pageX - slider.offsetLeft;
      scrollLeft = slider.scrollLeft;
      e.preventDefault();
    });

    slider.addEventListener("mouseleave", stopDrag);
    window.addEventListener("mouseup", stopDrag);

    slider.addEventListener("mousemove", function (e) {
      if (!isDown) return;
      var x = e.pageX - slider.offsetLeft;
      var walk = (x - startX) * 1.2;
      if (Math.abs(walk) > 4) moved = true;
      slider.scrollLeft = scrollLeft - walk;
    });

    slider.addEventListener("click", function (e) {
      if (moved) {
        e.preventDefault();
        e.stopPropagation();
      }
      moved = false;
    }, true);

    // suporte básico a pointer events
    if (window.PointerEvent) {
      slider.addEventListener("pointerdown", function (e) {
        if (e.pointerType !== "mouse") return;
        if (window.innerWidth <= 680) return;
        isDown = true;
        moved = false;
        slider.classList.add("dragging");
        startX = e.pageX - slider.offsetLeft;
        scrollLeft = slider.scrollLeft;
      });

      slider.addEventListener("pointermove", function (e) {
        if (!isDown || e.pointerType !== "mouse") return;
        var x = e.pageX - slider.offsetLeft;
        var walk = (x - startX) * 1.2;
        if (Math.abs(walk) > 4) moved = true;
        slider.scrollLeft = scrollLeft - walk;
      });

      slider.addEventListener("pointerup", stopDrag);
      slider.addEventListener("pointercancel", stopDrag);
    }
  }



  function initArchitectureScrollArrows() {
    var slider = document.querySelector(".architecture-cards");
    if (!slider) return;

    var parent = slider.parentElement;
    if (!parent) return;

    var shell = parent.querySelector(".architecture-scroll-shell");
    if (!shell) {
      shell = document.createElement("div");
      shell.className = "architecture-scroll-shell";
      parent.insertBefore(shell, slider);
      shell.appendChild(slider);
    }

    var prev = shell.querySelector(".arch-scroll-arrow.prev");
    var next = shell.querySelector(".arch-scroll-arrow.next");

    if (!prev) {
      prev = document.createElement("button");
      prev.className = "arch-scroll-arrow prev";
      prev.type = "button";
      prev.setAttribute("aria-label", "Scroll left");
      prev.innerHTML = '<svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M14.5 5L8 12l6.5 7" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
      shell.appendChild(prev);
    }

    if (!next) {
      next = document.createElement("button");
      next.className = "arch-scroll-arrow next";
      next.type = "button";
      next.setAttribute("aria-label", "Scroll right");
      next.innerHTML = '<svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M9.5 5L16 12l-6.5 7" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
      shell.appendChild(next);
    }

    function step() {
      return Math.max(260, Math.round(slider.clientWidth * 0.72));
    }

    function updateArrows() {
      var max = slider.scrollWidth - slider.clientWidth - 2;
      prev.disabled = slider.scrollLeft <= 2;
      next.disabled = slider.scrollLeft >= max;
    }

    prev.addEventListener("click", function () {
      slider.scrollBy({ left: -step(), behavior: "smooth" });
    });

    next.addEventListener("click", function () {
      slider.scrollBy({ left: step(), behavior: "smooth" });
    });

    slider.addEventListener("scroll", updateArrows, { passive: true });
    window.addEventListener("resize", updateArrows);
    setTimeout(updateArrows, 80);
    updateArrows();
  }


  function bindEvents() {
    var languageButton = $("#language-button");
    if (languageButton) {
      languageButton.addEventListener("click", function () {
        var menu = $("#language-menu");
        if (menu) menu.classList.toggle("open");
      });
    }

    document.addEventListener("click", function (event) {
      if (!event.target.closest(".language-switcher")) {
        var menu = $("#language-menu");
        if (menu) menu.classList.remove("open");
      }
    });

    initTheme();

    var enter = $("#enter-btn");
    if (enter) {
      enter.addEventListener("click", function () {
        var boot = $("#boot");
        if (boot) boot.classList.add("hidden");
      });
    }
  }

  function render() {
    document.documentElement.lang = currentLang === "pt" ? "pt-PT" : "en";
    renderLanguageMenu();
    renderNav();
    renderHero();
    renderStats();
    renderContent();
    renderArchitectureCards();
    renderLayers();
    renderModules();
    renderTechExamplesInstall();
    renderFounderPage();
    revealOnScroll();
  }

  document.addEventListener("DOMContentLoaded", function () {
    try {
      initTheme();
      render();
      bindEvents();
      bootSequence();
      initCursor();
      initArchitectureDragScroll();
      initArchitectureScrollArrows();
      initAnchorOffset();
    } catch (error) {
      console.error("LightQOS page error:", error);
      var boot = $("#boot");
      if (boot) boot.classList.add("hidden");
      $$(".reveal").forEach(function (el) { el.classList.add("visible"); });
    }
  });
})();
