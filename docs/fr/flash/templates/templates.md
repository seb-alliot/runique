# Affichage dans les templates

## Tag automatique {% messages %}

La balise `{% messages %}` affiche automatiquement tous les messages :

```html
{% messages %}
```

Elle inclut le template interne `message/message_include.html` qui génère :

```html
{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

---

## Placement recommandé

Placez `{% messages %}` dans votre template de base, juste avant le contenu principal :

```html
<!-- base.html -->
<body>
    <header>...</header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>...</footer>
</body>
```

---

## Personnalisation de l'affichage

Pour personnaliser l'affichage, bouclez manuellement sur `messages` :

```html
{% if messages %}
    {% for msg in messages %}
        <div class="alert alert-{{ msg.level }}" role="alert">
            <strong>
                {% if msg.level == "success" %}✅
                {% elif msg.level == "error" %}❌
                {% elif msg.level == "warning" %}⚠️
                {% elif msg.level == "info" %}ℹ️
                {% endif %}
            </strong>
            {{ msg.content }}
        </div>
    {% endfor %}
{% endif %}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/flash/macros) | Toutes les macros flash |
| [Handlers](/docs/fr/flash/handlers) | Utilisation dans les handlers |

## Retour au sommaire

- [Flash Messages](/docs/fr/flash)
