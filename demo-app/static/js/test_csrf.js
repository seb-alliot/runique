// async function testCsrfLogic() {
//     // 1. Récupérer le jeton initial CSRF depuis le DOM
//     const csrfInput = document.querySelector('input[name="csrf_token"]');
//     const initialToken = csrfInput ? csrfInput.value : null;

//     console.log("=== Début des tests CSRF ===");
//     console.log("Jeton initial détecté :", initialToken);

//     if (!initialToken) {
//         console.error("❌ Erreur : Aucun jeton trouvé sur la page. Vérifiez votre balise {% csrf_token %}.");
//         return;
//     }

//     // --- TEST 1 : Requête sans jeton , sa dois échouer ---
//     console.log("\nTest 1 : Envoi d'un POST sans header CSRF...");
//     try {
//         const res1 = await fetch('/test-csrf', {
//             method: 'POST',
//             body: JSON.stringify({ data: "test" })
//         });
//         console.log(res1.status === 403
//             ? "✅ Succès du test : Le serveur a bien bloqué la requête (403)."
//             : "❌ Échec du test : Le serveur aurait dû bloquer (Status: " + res1.status + ")");
//     } catch (e) { console.error("Erreur réseau :", e); }

//     // --- TEST 2 : Requête avec jeton valide ---
//     console.log("\nTest 2 : Envoi d'un POST avec jeton valide...");
//     try {
//         const res2 = await fetch('/test-csrf', {
//             method: 'POST',
//             headers: {
//                 'Content-Type': 'application/json',
//                 'X-CSRF-Token': initialToken
//             },
//             body: JSON.stringify({ data: "test" })
//         });

//         if (res2.ok) {
//             console.log("✅ Succès du test : Requête acceptée (200/201).");

//             // Vérification de la rotation dans le header de la réponse
//             const newToken = res2.headers.get('X-CSRF-Token');
//             if (newToken && newToken !== initialToken) {
//                 console.log("✅ Rotation confirmée : Nouveau jeton reçu =", newToken);
//             } else {
//                 console.warn("⚠️ Attention : Le jeton n'a pas changé ou le header X-CSRF-Token est absent de la réponse.");
//             }
//         } else {
//             console.error("❌ Échec du test : Requête refusée avec le bon jeton (Status: " + res2.status + ")");
//         }
//     } catch (e) { console.error("Erreur réseau :", e); }
// }

// testCsrfLogic();