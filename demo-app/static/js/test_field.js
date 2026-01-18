async function testCsrfForm() {
    console.log("=== Début des tests CSRF avec Credentials ===");

    const csrfInput = document.querySelector('input[name="csrf_token"]');
    const initialToken = csrfInput ? csrfInput.value : null;
    console.log("Jeton initial :", initialToken);

    if (!initialToken) {
        console.error("❌ Aucun jeton CSRF trouvé dans le DOM.");
        return;
    }

    // --- TEST 1 : POST JSON sans token ---
    console.log("\nTest 1 : POST JSON sans token...");
    try {
        const res1 = await fetch('/test-fields', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'same-origin', // Envoie le cookie de session
            body: JSON.stringify({ test: "data" })
        });
        console.log(res1.status === 403
            ? "✅ Bloqué comme attendu (403)"
            : "❌ Devait être bloqué, status: " + res1.status);
    } catch(e) { console.error(e); }

    // --- TEST 2 : POST JSON avec token ---
    console.log("\nTest 2 : POST JSON avec token...");
    try {
        const res2 = await fetch('/test-fields', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': initialToken
            },
            credentials: 'same-origin', // CRITIQUE : Envoie le cookie id=...
            body: JSON.stringify({ test: "data" })
        });
        console.log(res2.ok ? "✅ Accepté (200/201)" : "❌ Refusé, status: " + res2.status);

        const newToken = res2.headers.get('X-CSRF-Token');
        if (newToken) console.log("Nouveau token reçu pour rotation.");
    } catch(e) { console.error(e); }

    // --- TEST 3 : multipart/form-data ---
    console.log("\nTest 3 : multipart/form-data avec token...");
    try {
        const formData = new FormData();
        formData.append('phone', '0612345678');
        formData.append('csrf_token', initialToken);

        const res3 = await fetch('/test-fields', {
            method: 'POST',
            headers: { 'X-CSRF-Token': initialToken },
            credentials: 'same-origin', // CRITIQUE
            body: formData
        });
        console.log(res3.ok ? "✅ Accepté (200/201)" : "❌ Refusé, status: " + res3.status);
    } catch(e) { console.error(e); }

    console.log("=== Fin des tests CSRF ===");
}

testCsrfForm();