/* ==========================================================================
   Warpai Interactive Site Scripts
   Adapted from herdr.dev for Warpai Terminal
   ========================================================================== */

(function () {
  /* 1. Ground Mode Management (ink vs paper) */
  window.warpaiGround = {
    read: function () {
      try {
        return localStorage.getItem("warpai-theme") === "paper" ? "paper" : "ink";
      } catch (e) {
        return "ink";
      }
    },
    apply: function (mode) {
      var root = document.documentElement;
      root.setAttribute("data-mode", mode);
      root.setAttribute("data-palette", mode === "paper" ? "solarized-light" : "catppuccin");
      return mode;
    },
    save: function (mode) {
      try {
        localStorage.setItem("warpai-theme", mode);
      } catch (e) {}
    },
  };

  var groundButtons = document.querySelectorAll("[data-ground]");
  function paintGround(mode) {
    window.warpaiGround.apply(mode);
    groundButtons.forEach(function (b) {
      b.setAttribute("aria-pressed", String(b.dataset.ground === mode));
    });
  }

  groundButtons.forEach(function (b) {
    b.addEventListener("click", function () {
      window.warpaiGround.save(b.dataset.ground);
      paintGround(b.dataset.ground);
    });
  });

  paintGround(window.warpaiGround.read());

  /* 2. Mobile Nav Sheet */
  var menuBtn = document.getElementById("nav-menu");
  var sheet = document.getElementById("nav-sheet");
  if (menuBtn && sheet) {
    function setSheet(open) {
      sheet.classList.toggle("open", open);
      menuBtn.setAttribute("aria-expanded", String(open));
    }
    menuBtn.addEventListener("click", function (event) {
      event.stopPropagation();
      setSheet(!sheet.classList.contains("open"));
    });
    document.addEventListener("click", function (event) {
      if (!sheet.contains(event.target)) setSheet(false);
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") setSheet(false);
    });
  }

  /* 3. Typing Tape Animation */
  var out = document.querySelector("[data-tape]");
  if (out) {
    var LINES = [
      "warpai agent start reviewer --kind codex",
      "warpai worktree add feat/vt100-parser",
      "warpai agent wait reviewer --until waiting-approval",
      "warpai task integrate --task-id task-102",
      "warpai companion attach workbox-gpu --port 2222",
      'warpai agent prompt reviewer "Verify isolated worktree diff" --wait',
    ];

    if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
      out.textContent = LINES[0];
    } else {
      var line = 0, ch = 0, erasing = false, live = false, timer = null;

      function tick() {
        var text = LINES[line];
        if (!erasing) {
          ch++;
          out.textContent = text.slice(0, ch);
          if (ch >= text.length) {
            erasing = true;
            return schedule(2000);
          }
          return schedule(28 + Math.random() * 40);
        }
        ch -= 3;
        if (ch <= 0) {
          ch = 0;
          erasing = false;
          line = (line + 1) % LINES.length;
          out.textContent = "";
          return schedule(400);
        }
        out.textContent = text.slice(0, ch);
        return schedule(16);
      }

      function schedule(ms) {
        timer = setTimeout(function () {
          if (live) tick();
        }, ms);
      }

      new IntersectionObserver(function (entries) {
        entries.forEach(function (e) {
          if (e.isIntersecting && !live) {
            live = true;
            tick();
          } else if (!e.isIntersecting && live) {
            live = false;
            clearTimeout(timer);
          }
        });
      }, { threshold: 0.35 }).observe(out.closest(".tape"));
    }
  }

  /* 4. Copy Buttons */
  [["copy-btn", "install-cmd"], ["copy-btn-2", "install-cmd-2"]].forEach(function (pair) {
    var button = document.getElementById(pair[0]);
    var source = document.getElementById(pair[1]);
    if (!button || !source) return;
    button.addEventListener("click", function () {
      var text = source.textContent.replace(/^\s*\$\s*/, "").trim();
      if (!text) return;
      navigator.clipboard.writeText(text).then(function () {
        var was = button.textContent;
        button.textContent = "copied";
        setTimeout(function () { button.textContent = was; }, 1400);
      }).catch(function () {});
    });
  });

  // Package command copy buttons
  document.querySelectorAll(".pkg-cmd").forEach(function (item) {
    var btn = item.querySelector("button");
    var code = item.querySelector("code");
    if (!btn || !code) return;
    btn.addEventListener("click", function () {
      var text = item.getAttribute("data-copy") || code.textContent.trim();
      navigator.clipboard.writeText(text).then(function () {
        var prev = btn.textContent;
        btn.textContent = "copied";
        setTimeout(function () { btn.textContent = prev; }, 1400);
      }).catch(function () {});
    });
  });

  /* 5. Braille Spinner Animation */
  var spinnerFrames = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
  var spinnerTick = 0;
  setInterval(function () {
    spinnerTick = (spinnerTick + 1) % spinnerFrames.length;
    var els = document.querySelectorAll(".braille-spinner");
    for (var i = 0; i < els.length; i++) els[i].textContent = spinnerFrames[spinnerTick];
  }, 80);

  /* 6. Live Working Timer */
  var ccTimer = document.getElementById("cc-timer");
  if (ccTimer) {
    var seconds = 13 * 60 + 36;
    setInterval(function () {
      if (document.hidden) return;
      seconds += 1;
      if (seconds >= 18 * 60) seconds = 0;
      ccTimer.textContent = Math.floor(seconds / 60) + "m " + (seconds % 60) + "s";
    }, 1000);
  }

  /* 7. Interactive Terminal Mock (#herd-mock / #warpai-mock) */
  var mock = document.getElementById("herd-mock") || document.getElementById("warpai-mock");
  if (mock) {
    function syncAgents() {
      var view = mock.querySelector(".mock-view.active");
      var tab = view && view.querySelector(".mock-tabs span.active");
      var ws = view && view.getAttribute("data-view");
      var tabId = tab && tab.getAttribute("data-tab");
      mock.querySelectorAll(".mock-agent").forEach(function (a) {
        a.classList.toggle(
          "active",
          a.getAttribute("data-ws-link") === ws &&
            a.getAttribute("data-tab-link") === tabId
        );
      });
    }

    function selectTab(tab) {
      var target = tab.getAttribute("data-tab");
      if (!target) return;
      tab.parentElement.querySelectorAll("span").forEach(function (t) {
        t.classList.remove("active");
      });
      tab.classList.add("active");
      var view = tab.closest(".mock-view");
      view.querySelectorAll(".mock-panes").forEach(function (panes) {
        panes.classList.toggle("active", panes.getAttribute("data-tab") === target);
      });
      syncAgents();
    }

    function activateWorkspace(id, tabId) {
      mock.querySelectorAll(".mock-space").forEach(function (ws) {
        ws.classList.toggle("active", ws.getAttribute("data-ws") === id);
      });
      mock.querySelectorAll(".mock-view").forEach(function (view) {
        view.classList.toggle("active", view.getAttribute("data-view") === id);
      });
      if (tabId) {
        var view = mock.querySelector('.mock-view[data-view="' + id + '"]');
        var tab = view && view.querySelector('.mock-tabs span[data-tab="' + tabId + '"]');
        if (tab) selectTab(tab);
      }
      syncAgents();
    }

    mock.querySelectorAll(".mock-space").forEach(function (ws) {
      ws.addEventListener("click", function () {
        activateWorkspace(ws.getAttribute("data-ws"));
      });
    });

    mock.querySelectorAll(".mock-agent").forEach(function (a) {
      a.addEventListener("click", function () {
        activateWorkspace(
          a.getAttribute("data-ws-link"),
          a.getAttribute("data-tab-link")
        );
      });
    });

    mock.querySelectorAll(".mock-tabs span").forEach(function (tab) {
      if (!tab.getAttribute("data-tab")) return;
      tab.addEventListener("click", function (event) {
        event.stopPropagation();
        selectTab(tab);
      });
    });
  }

  /* 8. Metric Numbers Counter Animation */
  var fmt = function (n) { return n.toLocaleString("en-US"); };
  var reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

  function settle(el, target) {
    if (!el || typeof target !== "number") return;
    var from = Number(String(el.textContent).replace(/[^0-9]/g, "")) || 0;
    if (reduced || !from || from === target) { el.textContent = fmt(target); return; }
    var started = null, span = target - from;
    requestAnimationFrame(function step(now) {
      if (started === null) started = now;
      var t = Math.min(1, (now - started) / 900);
      var eased = 1 - Math.pow(1 - t, 3);
      el.textContent = fmt(Math.round(from + span * eased));
      if (t < 1) requestAnimationFrame(step);
    });
  }

  // Dynamic GitHub Stars Fetch for OthinusG/warpai
  function fetchWarpaiStars() {
    fetch("https://api.github.com/repos/OthinusG/warpai")
      .then(function (res) {
        if (!res.ok) throw new Error("GitHub API status " + res.status);
        return res.json();
      })
      .then(function (data) {
        if (data && typeof data.stargazers_count === "number") {
          var count = data.stargazers_count;
          document.querySelectorAll("[data-stars]").forEach(function (el) {
            settle(el, count);
          });
        }
      })
      .catch(function () {});
  }
  fetchWarpaiStars();
})();
