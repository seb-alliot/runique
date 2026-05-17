(function () {
  "use strict";

  const mainGrid   = document.getElementById("bulk-fields-main");
  const permGrid   = document.getElementById("bulk-perm-fields");
  const permsCard  = document.getElementById("bulk-perms-section");
  const btnFields  = document.getElementById("bulk-toggle-fields");
  const btnPerms   = document.getElementById("bulk-toggle-perms");

  // Move all .boolean-field elements into the permissions section.
  if (mainGrid && permGrid && permsCard) {
    const boolFields = Array.from(mainGrid.querySelectorAll(".boolean-field"));
    if (boolFields.length > 0) {
      boolFields.forEach(function (f) { permGrid.appendChild(f); });
      permsCard.hidden = false;
    }
  }

  function toggleAll(container, btn, labelOn, labelOff) {
    if (!container || !btn) return;
    btn.addEventListener("click", function () {
      const cbs = Array.from(container.querySelectorAll('input[type="checkbox"]'));
      const allChecked = cbs.every(function (cb) { return cb.checked; });
      cbs.forEach(function (cb) { cb.checked = !allChecked; });
      btn.textContent = allChecked ? labelOn : labelOff;
    });
  }

  toggleAll(mainGrid,  btnFields, "Tout cocher",              "Tout décocher");
  toggleAll(permGrid,  btnPerms,  "Tout cocher les permissions", "Tout décocher les permissions");
})();
