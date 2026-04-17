(function () {
  "use strict";

  function init() {
    const checkAll = document.getElementById("bulk-check-all");
    const bar = document.getElementById("bulk-action-bar");
    const countEl = document.getElementById("bulk-count");
    const idsInput = document.getElementById("bulk-ids-input");
    const actionInput = document.getElementById("bulk-action-input");
    const bulkForm = document.getElementById("bulk-form");

    if (!bar || !bulkForm) return;

    function getChecked() {
      return Array.from(document.querySelectorAll(".bulk-check:checked"));
    }

    function updateBar() {
      const checked = getChecked();
      const n = checked.length;
      if (countEl) countEl.textContent = n;
      bar.classList.toggle("bulk-bar-visible", n > 0);
    }

    function collectIds() {
      return getChecked().map((cb) => cb.value).join(",");
    }

    if (checkAll) {
      checkAll.addEventListener("change", function () {
        document
          .querySelectorAll(".bulk-check")
          .forEach((cb) => (cb.checked = checkAll.checked));
        updateBar();
      });
    }

    document.querySelectorAll(".bulk-check").forEach((cb) => {
      cb.addEventListener("change", function () {
        if (checkAll && !this.checked) checkAll.checked = false;
        updateBar();
      });
    });

    bulkForm.addEventListener("submit", function (e) {
      idsInput.value = collectIds();
      if (!idsInput.value) {
        e.preventDefault();
        return;
      }
    });

    document.querySelectorAll("[data-bulk-action]").forEach((btn) => {
      btn.addEventListener("click", function (e) {
        const action = this.dataset.bulkAction;
        const n = getChecked().length;
        if (action === "delete") {
          if (!confirm("Supprimer " + n + " élément(s) ? Cette action est irréversible.")) {
            e.preventDefault();
            return;
          }
        }
        actionInput.value = action;
        idsInput.value = collectIds();
        bulkForm.submit();
      });
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
