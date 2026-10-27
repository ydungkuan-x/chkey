// SHA-256 Constant
      const RELEASE_SHA256 =
        "ECFF39F3358CC29307CE77AE5AA8B1653010998E08574ED74F87807A97061EA8";

      // Toast helper
      function showToast(message, icon = "✅") {
        const toast = document.getElementById("toast");
        const msg = document.getElementById("toast-msg");
        const ico = document.getElementById("toast-icon");
        if (!toast) return;

        msg.innerText = message;
        ico.innerText = icon;
        toast.classList.add("show");

        setTimeout(() => {
          toast.classList.remove("show");
        }, 3000);
      }

      // Copy checksum
      function copyChecksum() {
        navigator.clipboard
          .writeText(RELEASE_SHA256)
          .then(() => {
            showToast("Đã chép mã băm SHA-256 vào clipboard!", "📋");
          })
          .catch(() => {
            showToast(RELEASE_SHA256, "📋");
          });
      }

      const copyShaBtn = document.getElementById("copy-sha-btn");
      if (copyShaBtn) {
        copyShaBtn.addEventListener("click", copyChecksum);
      }
