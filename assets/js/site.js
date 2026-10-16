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

  /* 7. Interactive Warpai 3-Column Workbench Mock (#herd-mock / #warpai-mock) */
  var mock = document.getElementById("herd-mock") || document.getElementById("warpai-mock");
  if (mock) {
    var vtabs = mock.querySelectorAll(".mock-vtab");
    var screens = mock.querySelectorAll(".mock-term-screen");
    var agentItems = mock.querySelectorAll(".mock-agent-item");
    var focusTags = mock.querySelectorAll(".mock-focus-tag");
    var modeBtns = mock.querySelectorAll(".mock-pill-btn");
    var worktreeView = mock.querySelector("#mock-view-worktree");
    var projectView = mock.querySelector("#mock-view-project");
    var coordTrigger = mock.querySelector("#mock-coord-trigger");
    var coordMenu = mock.querySelector("#mock-coord-menu");
    var coordName = mock.querySelector("#mock-coord-active-name");
    var coordOptions = mock.querySelectorAll(".mock-coord-option");
    var btnNewWorktree = mock.querySelector("#mock-btn-new-worktree");
    var btnExplorer = mock.querySelector("#mock-btn-explorer");
    var fileDrawer = mock.querySelector("#mock-file-drawer");
    var closeDrawer = mock.querySelector("#mock-close-drawer");
    var chipNotify = mock.querySelector("#mock-chip-notify");
    var btnToggleSidebar = mock.querySelector("#mock-btn-toggle-sidebar");
    var leftSidebar = mock.querySelector("#mock-left-sidebar");
    var btnToggleCollab = mock.querySelector("#mock-btn-toggle-collab");
    var rightPanel = mock.querySelector("#mock-right-panel");
    var searchInput = mock.querySelector("#mock-search-input");
    var btnTheme = mock.querySelector("#mock-theme-toggle-btn");
    var btnSendAgent = mock.querySelector("#mock-btn-send-agent");
    var btnHistory = mock.querySelector("#mock-btn-history");
    var tasksList = mock.querySelector("#mock-tasks-list");
    var btnRefreshQuota = mock.querySelector("#mock-btn-refresh-quota");
    var btnNewTab = mock.querySelector("#mock-btn-new-tab");

    // 1. Switch Vertical Tab
    function switchTab(agentId) {
      if (!agentId) return;
      vtabs.forEach(function (t) {
        t.classList.toggle("active", t.getAttribute("data-agent") === agentId);
      });
      screens.forEach(function (s) {
        s.classList.toggle("active", s.getAttribute("data-screen") === agentId);
      });
      agentItems.forEach(function (item) {
        item.classList.toggle("active", item.getAttribute("data-agent") === agentId);
      });
    }

    vtabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        switchTab(tab.getAttribute("data-agent"));
      });
    });

    agentItems.forEach(function (item) {
      item.addEventListener("click", function () {
        switchTab(item.getAttribute("data-agent"));
      });
    });

    focusTags.forEach(function (tag) {
      tag.addEventListener("click", function () {
        switchTab(tag.getAttribute("data-focus"));
      });
    });

    // 2. Collaboration Mode Switcher (Project vs Worktree)
    var isZh = document.documentElement.lang === "zh";
    var defaultEmptyTasksText = isZh ? "当前工作区无待办任务。" : "No tasks on this page.";
    var projectTasksHtml = isZh
      ? '<div style="display:flex;flex-direction:column;gap:4px;margin-bottom:6px;">' +
        '<div style="font-size:9.5px;padding:3px 6px;background:var(--mock-border-subtle);border-radius:3px;"><span style="color:#d97757;font-weight:700;">任务 #2942</span> 行业数据回归分析 (执行中)</div>' +
        '<div style="font-size:9.5px;padding:3px 6px;background:var(--mock-border-subtle);border-radius:3px;"><span style="color:#3fb950;font-weight:700;">任务 #2943</span> 论文初稿交叉审稿 (待审批)</div>' +
        '</div>'
      : '<div style="display:flex;flex-direction:column;gap:4px;margin-bottom:6px;">' +
        '<div style="font-size:9.5px;padding:3px 6px;background:var(--mock-border-subtle);border-radius:3px;"><span style="color:#d97757;font-weight:700;">Task #2942</span> Data regression &amp; Sharpe ratio (Running)</div>' +
        '<div style="font-size:9.5px;padding:3px 6px;background:var(--mock-border-subtle);border-radius:3px;"><span style="color:#3fb950;font-weight:700;">Task #2943</span> Manuscript peer review (Pending)</div>' +
        '</div>';

    modeBtns.forEach(function (btn) {
      btn.addEventListener("click", function () {
        var mode = btn.getAttribute("data-mode");
        modeBtns.forEach(function (b) { b.classList.toggle("active", b === btn); });
        if (mode === "project") {
          if (worktreeView) worktreeView.classList.remove("active");
          if (projectView) projectView.classList.add("active");
          if (tasksList) tasksList.innerHTML = projectTasksHtml;
        } else {
          if (projectView) projectView.classList.remove("active");
          if (worktreeView) worktreeView.classList.add("active");
          if (tasksList) tasksList.innerHTML = '<div class="mock-tasks-empty-text">' + defaultEmptyTasksText + '</div>';
        }
      });
    });

    // 3. Coordinator Dropdown
    if (coordTrigger && coordMenu) {
      coordTrigger.addEventListener("click", function (e) {
        e.stopPropagation();
        coordMenu.classList.toggle("open");
      });
      coordOptions.forEach(function (opt) {
        opt.addEventListener("click", function (e) {
          e.stopPropagation();
          coordOptions.forEach(function (o) { o.classList.remove("active"); });
          opt.classList.add("active");
          if (coordName) coordName.textContent = opt.textContent.trim();
          coordMenu.classList.remove("open");
        });
      });
      document.addEventListener("click", function () {
        coordMenu.classList.remove("open");
      });
    }

    // 4. File Explorer Drawer
    if (btnExplorer && fileDrawer) {
      btnExplorer.addEventListener("click", function () {
        fileDrawer.classList.toggle("open");
      });
    }
    if (closeDrawer && fileDrawer) {
      closeDrawer.addEventListener("click", function () {
        fileDrawer.classList.remove("open");
      });
    }

    // 5. Dismiss Notification Chip
    if (chipNotify) {
      chipNotify.addEventListener("click", function () {
        chipNotify.style.opacity = "0";
        setTimeout(function () { chipNotify.style.display = "none"; }, 150);
      });
    }

    // 6. Sidebar & Panel Toggles
    if (btnToggleSidebar && leftSidebar) {
      btnToggleSidebar.addEventListener("click", function () {
        leftSidebar.classList.toggle("collapsed");
      });
    }
    if (btnToggleCollab && rightPanel) {
      btnToggleCollab.addEventListener("click", function () {
        rightPanel.classList.toggle("collapsed");
      });
    }

    // 7. Theme Toggle in Mock Window
    if (btnTheme && window.warpaiGround) {
      btnTheme.addEventListener("click", function () {
        var current = window.warpaiGround.read();
        var next = current === "paper" ? "ink" : "paper";
        window.warpaiGround.save(next);
        window.warpaiGround.apply(next);
      });
    }

    // 8. Search Tabs Filter
    if (searchInput) {
      searchInput.addEventListener("input", function () {
        var q = searchInput.value.toLowerCase().trim();
        vtabs.forEach(function (tab) {
          var text = tab.textContent.toLowerCase();
          tab.style.display = !q || text.indexOf(q) !== -1 ? "flex" : "none";
        });
      });
    }

    // 9. Add New Worktree simulation
    if (btnNewWorktree && worktreeView) {
      btnNewWorktree.addEventListener("click", function () {
        var existing = worktreeView.querySelector("#mock-dynamic-worktree");
        if (existing) {
          existing.remove();
        } else {
          var div = document.createElement("div");
          div.id = "mock-dynamic-worktree";
          div.className = "mock-worktree-node";
          div.style.marginTop = "6px";
          div.innerHTML = '<div class="mock-worktree-pill active" style="border-color:var(--mock-green);">' +
            '<span class="mock-arrow">▶</span> feat/new-agent-spec <small style="color:var(--mock-green);font-size:9px;">[new]</small></div>' +
            '<div class="mock-worktree-agents"><div class="mock-agent-item active" style="color:var(--mock-green);">● claude-worker (ready)</div></div>';
          worktreeView.appendChild(div);
        }
      });
    }

    // 10. Send to Agent simulation
    if (btnSendAgent) {
      btnSendAgent.addEventListener("click", function () {
        var activeScreen = mock.querySelector(".mock-term-screen.active");
        if (activeScreen) {
          var promptRow = activeScreen.querySelector(".mock-prompt-row");
          if (promptRow) {
            var msg = document.createElement("div");
            msg.className = "mock-term-line";
            msg.style.color = "var(--mock-blue)";
            msg.textContent = isZh ? "✔ 已向智能体输入流投递 warp_agent_send 任务。" : "✔ Dispatched warp_agent_send task to agent stdin.";
            activeScreen.insertBefore(msg, promptRow);
            setTimeout(function () { msg.remove(); }, 3500);
          }
        }
      });
    }

    // 11. History Toggle
    var showingHistory = false;
    if (btnHistory && tasksList) {
      btnHistory.addEventListener("click", function () {
        showingHistory = !showingHistory;
        if (showingHistory) {
          tasksList.innerHTML = isZh
            ? '<div style="display:flex;flex-direction:column;gap:3px;font-size:9px;color:var(--mock-muted);">' +
              '<div><b style="color:var(--mock-green);">✔ #2940</b> 分支初始化 (已合并)</div>' +
              '<div><b style="color:var(--mock-green);">✔ #2941</b> VT100 解析器集成 (已验收)</div>' +
              '</div>'
            : '<div style="display:flex;flex-direction:column;gap:3px;font-size:9px;color:var(--mock-muted);">' +
              '<div><b style="color:var(--mock-green);">✔ #2940</b> Branch bootstrap (Merged)</div>' +
              '<div><b style="color:var(--mock-green);">✔ #2941</b> VT100 observer (Accepted)</div>' +
              '</div>';
        } else {
          tasksList.innerHTML = '<div class="mock-tasks-empty-text">' + defaultEmptyTasksText + '</div>';
        }
      });
    }

    // 12. Quota Refresh
    if (btnRefreshQuota) {
      btnRefreshQuota.addEventListener("click", function () {
        btnRefreshQuota.style.transform = "rotate(360deg)";
        btnRefreshQuota.style.transition = "transform 0.5s ease";
        setTimeout(function () {
          btnRefreshQuota.style.transform = "none";
          btnRefreshQuota.style.transition = "none";
        }, 500);
      });
    }

    // 13. New Tab
    if (btnNewTab) {
      btnNewTab.addEventListener("click", function () {
        var count = vtabs.length + 1;
        var newTab = document.createElement("div");
        newTab.className = "mock-vtab";
        newTab.setAttribute("data-agent", "agent-" + count);
        newTab.innerHTML = '<div class="mock-vtab-badge mock-badge-warpai">✦</div>' +
          '<div class="mock-vtab-content"><div class="mock-vtab-name">tab-' + count + '</div>' +
          '<div class="mock-vtab-branch"><svg viewBox="0 0 16 16" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 6v4M4 8a4 4 0 0 1 4-4h2"/></svg> main</div></div>';
        mock.querySelector("#mock-vtabs-list").appendChild(newTab);
        newTab.addEventListener("click", function () {
          switchTab("claude");
        });
      });
    }
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
