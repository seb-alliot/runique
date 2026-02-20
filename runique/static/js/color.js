/**
 * Initialise les interactions entre le sélecteur de couleur (input type="color")
 * et le champ texte hexadécimal.
 */
document.addEventListener('input', function (event) {
    // Vérifie si l'élément qui a déclenché l'événement est un color-picker
    if (event.target.classList.contains('color-picker')) {
        const picker = event.target;
        const container = picker.closest('.color-picker-container');
        const textInput = container.querySelector('.color-text-input');

        if (textInput) {
            textInput.value = picker.value.toUpperCase();
        }
    }
    
    // Optionnel : Mise à jour du picker si on tape dans le champ texte
    if (event.target.classList.contains('color-text-input')) {
        const textInput = event.target;
        const container = textInput.closest('.color-picker-container');
        const picker = container.querySelector('.color-picker');
        
        // Vérifie si le format hex est valide avant de mettre à jour le picker
        if (picker && /^#[0-9A-F]{6}$/i.test(textInput.value)) {
            picker.value = textInput.value;
        }
    }
});