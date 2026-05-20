(function () {
  "use strict";

  function getChecked() {
    return Array.from(document.querySelectorAll(".admin-table__bulk-check:checked"));
  }

  function collectIds() {
    return getChecked().map((cb) => cb.value).join(",");
  }

  // Always look up live elements to avoid stale references after HTMX swaps.
  const getBulkForm    = () => document.getElementById("bulk-form");
  const getIdsInput    = () => document.getElementById("bulk-ids-input");
  const getActionInput = () => document.getElementById("bulk-action-input");

  const updateBar = function () {
    const bar = document.getElementById("bulk-action-bar");
    const countEl = document.getElementById("bulk-count");
    if (!bar) return;
    const n = getChecked().length;
    if (countEl) countEl.textContent = n;
    bar.classList.toggle("admin-group-action__bar--visible", n > 0);
  }

  const bindCheckboxes = function () {
    const checkAll = document.getElementById("bulk-check-all");
    if (checkAll) {
      checkAll.addEventListener("change", function () {
        document
          .querySelectorAll(".admin-table__bulk-check")
          .forEach((cb) => (cb.checked = checkAll.checked));
        updateBar();
      });
    }

    document.querySelectorAll(".admin-table__bulk-check").forEach((cb) => {
      cb.addEventListener("change", function () {
        const all = document.getElementById("bulk-check-all");
        if (all && !this.checked) all.checked = false;
        updateBar();
      });
    });

    updateBar();
  }

  // Event delegation on document — survives HTMX swaps.
  document.addEventListener("click", function (e) {
    const btn = e.target.closest("[data-bulk-action]");
    if (!btn) return;

    const bulkForm = getBulkForm();
    const idsInput = getIdsInput();
    const actionInput = getActionInput();
    if (!bulkForm || !idsInput || !actionInput) return;

    const action = btn.dataset.bulkAction;
    const n = getChecked().length;

    if (action === "delete") {
      e.preventDefault();
      const msg = "Supprimer " + n + " élément(s) ? Cette action est irréversible.";
      const doDelete = () => {
        actionInput.value = "delete";
        idsInput.value = collectIds();
        bulkForm.submit();
      };
      if (typeof window.runique_confirm === "function") {
        window.runique_confirm(msg, { okLabel: "Supprimer", cancelLabel: "Annuler" }).then(
          (ok) => { if (ok) doDelete(); }
        );
      } else if (window.confirm(msg)) {
        doDelete();
      }
      return;
    }

    if (action === "bulk-edit") {
      const ids = collectIds();
      if (!ids) return;
      const base = bulkForm.action.replace(/\/bulk$/, "/bulk");
      window.location.href = base + "?ids=" + encodeURIComponent(ids);
      return;
    }

    actionInput.value = action;
    idsInput.value = collectIds();
    bulkForm.submit();
  });

  // Après chaque swap HTMX, rebind les checkboxes recréées
  document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target && e.target.id === "list-content") {
      bindCheckboxes();
    }
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bindCheckboxes);
  } else {
    bindCheckboxes();
  }
})();
