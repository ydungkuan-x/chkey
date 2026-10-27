// Auto-detect OS to highlight appropriate download button
      (function detectOS() {
        const isMac =
          /Macintosh|MacIntel|MacPPC|Mac68K|Mac/i.test(navigator.userAgent) ||
          (navigator.platform &&
            navigator.platform.toUpperCase().indexOf("MAC") >= 0);
        if (isMac) {
          switchGuide("mac");
          const chipWin = document.getElementById("chip-win");
          const chipMacArm = document.getElementById("chip-mac-arm");
          const macBtn = document.getElementById("hero-download-mac-btn");
          const winBtn = document.getElementById("hero-download-btn");
          const cardWin = document.getElementById("card-win-x64");
          const cardMac = document.getElementById("card-mac-arm");

          if (chipWin) chipWin.classList.remove("active");
          if (chipMacArm) chipMacArm.classList.add("active");
          if (cardWin) cardWin.classList.remove("primary-hub");
          if (cardMac) cardMac.classList.add("primary-hub");

          if (macBtn && winBtn) {
            macBtn.style.order = "-1";
            macBtn.style.boxShadow = "0 10px 30px rgba(56, 189, 248, 0.4)";
            macBtn.style.borderColor = "#38bdf8";
          }

          const badge = document.querySelector(".badge-pill");
          if (badge) {
            badge.innerHTML = `<span class="badge-dot"></span> Đã nhận diện hệ điều hành: macOS • Bản v1.0.0`;
          }
        }
      })();
