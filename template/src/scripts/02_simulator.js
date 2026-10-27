/* ==========================================================================
       CHKEY TELEX IN-BROWSER SIMULATOR ENGINE
       Implements exact rules matching chkey_engine (Êđê / Tây Nguyên)
       ========================================================================== */
      const simInput = document.getElementById("sim-input");
      const charCount = document.getElementById("char-count");

      // Mapping table for 2-key and 3-key combinations
      const TELEX_MAP = {
        // Phụ âm
        bb: "ƀ",
        BB: "Ƀ",
        Bb: "ƀ",
        bB: "ƀ",
        cx: "č",
        CX: "Č",
        Cx: "č",
        cX: "č",
        nn: "ñ",
        NN: "Ñ",
        Nn: "ñ",
        nN: "ñ",
        dd: "đ",
        DD: "Đ",
        Dd: "đ",
        dD: "đ",

        // Nguyên âm có dấu trăng (Breve)
        ax: "ă",
        AX: "Ă",
        Ax: "ă",
        aX: "ă",
        ex: "ĕ",
        EX: "Ĕ",
        Ex: "ĕ",
        eX: "ĕ",
        ix: "ĭ",
        IX: "Ĭ",
        Ix: "ĭ",
        iX: "ĭ",
        ox: "ŏ",
        OX: "Ŏ",
        Ox: "ŏ",
        oX: "ŏ",
        ux: "ŭ",
        UX: "Ŭ",
        Ux: "ŭ",
        uX: "ŭ",

        // Nguyên âm nón (Circumflex)
        aa: "â",
        AA: "Â",
        Aa: "â",
        aA: "â",
        ee: "ê",
        EE: "Ê",
        Ee: "ê",
        eE: "ê",
        oo: "ô",
        OO: "Ô",
        Oo: "ô",
        oO: "ô",

        // Nguyên âm móc (Horn)
        ow: "ơ",
        OW: "Ơ",
        Ow: "ơ",
        oW: "ơ",
        uw: "ư",
        UW: "Ư",
        Uw: "ư",
        uW: "ư",
        uu: "ư",
        UU: "Ư",
        Uu: "ư",
        uU: "ư",

        // Nguyên âm đôi trăng kết hợp
        âx: "â̆",
        Âx: "Â̆",
        âX: "â̆",
        ÂX: "Â̆",
        êx: "ê̆",
        Êx: "Ê̆",
        êX: "ê̆",
        ÊX: "Ê̆",
        ôx: "ô̆",
        Ôx: "Ô̆",
        ôX: "ô̆",
        ÔX: "Ô̆",
        ơx: "ơ̆",
        Ơx: "Ơ̆",
        ơX: "ơ̆",
        ƠX: "Ơ̆",
        ưx: "ư̆",
        Ưx: "Ư̆",
        ưX: "ư̆",
        ƯX: "Ư̆",
      };

      if (simInput) {
        simInput.addEventListener("input", function (e) {
          let val = simInput.value;
          let pos = simInput.selectionStart;

          // Check 2-letter combos right before cursor
          if (pos >= 2) {
            let slice2 = val.substring(pos - 2, pos);
            if (TELEX_MAP[slice2]) {
              let repl = TELEX_MAP[slice2];
              simInput.value =
                val.substring(0, pos - 2) + repl + val.substring(pos);
              simInput.selectionStart = simInput.selectionEnd = pos - 1;
              updateCharCount();
              return;
            }
          }

          // Check 3-letter combos (e.g. aax -> â̆, eex -> ê̆, oox -> ô̆, owx -> ơ̆, uwx -> ư̆)
          if (pos >= 3) {
            let slice3 = val.substring(pos - 3, pos);
            const THREE_MAP = {
              aax: "â̆",
              AAX: "Â̆",
              Aax: "â̆",
              eex: "ê̆",
              EEX: "Ê̆",
              Eex: "ê̆",
              oox: "ô̆",
              OOX: "Ô̆",
              Oox: "ô̆",
              owx: "ơ̆",
              OWX: "Ơ̆",
              Owx: "ơ̆",
              uwx: "ư̆",
              UWX: "Ư̆",
              Uwx: "ư̆",
            };
            if (THREE_MAP[slice3]) {
              let repl = THREE_MAP[slice3];
              simInput.value =
                val.substring(0, pos - 3) + repl + val.substring(pos);
              simInput.selectionStart = simInput.selectionEnd = pos - 2;
              updateCharCount();
              return;
            }
          }

          updateCharCount();
        });
      }

      function updateCharCount() {
        if (!simInput || !charCount) return;
        charCount.innerText = `${simInput.value.length} ký tự`;
      }

      // Insert sample into simulator
      function insertSample(text) {
        if (!simInput) return;
        simInput.value = text + " ";
        simInput.focus();
        updateCharCount();
        showToast(`Đã nạp mẫu: "${text}"`, "✍️");
      }

      // Clear simulator
      function clearSim() {
        if (!simInput) return;
        simInput.value = "";
        simInput.focus();
        updateCharCount();
      }

      // Insert single character from virtual palette
      function insertChar(ch) {
        if (!simInput) return;
        const start = simInput.selectionStart || simInput.value.length;
        const end = simInput.selectionEnd || simInput.value.length;
        const text = simInput.value;
        simInput.value = text.substring(0, start) + ch + text.substring(end);
        simInput.selectionStart = simInput.selectionEnd = start + ch.length;
        simInput.focus();
        updateCharCount();
        showToast(`Đã chèn ký tự: ${ch}`, "✨");
      }
