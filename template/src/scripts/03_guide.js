// Switch Guide Tabs between Windows and macOS
      function switchGuide(os) {
        const winTab = document.getElementById("tab-guide-win");
        const macTab = document.getElementById("tab-guide-mac");
        const winSteps = document.getElementById("guide-win-steps");
        const macSteps = document.getElementById("guide-mac-steps");

        if (!winTab || !macTab || !winSteps || !macSteps) return;

        if (os === "mac") {
          winTab.classList.remove("active");
          winTab.setAttribute("aria-selected", "false");
          macTab.classList.add("active");
          macTab.setAttribute("aria-selected", "true");
          winSteps.classList.remove("active");
          macSteps.classList.add("active");
        } else {
          macTab.classList.remove("active");
          macTab.setAttribute("aria-selected", "false");
          winTab.classList.add("active");
          winTab.setAttribute("aria-selected", "true");
          macSteps.classList.remove("active");
          winSteps.classList.add("active");
        }
      }
