# Design References

Sources HTML des maquettes Claude Design utilisées comme référence visuelle 
et structurelle pour le développement.

## Lecture des fichiers

Les HTML peuvent être ouverts directement dans un navigateur pour visualiser 
les écrans tels qu'ils ont été conçus. Le code source contient les valeurs 
exactes du design system (couleurs, espacements, SVG des canards, etc.) 
qui doivent être réutilisées dans le code Vue.

## Inventaire

### The Pond - Design System.html
**Statut** : référence canonique du design system v0.2
**Type** : planche système (palette, typographie, composants, canards)
**Usage** : point de départ pour tokens.css et tailwind.config.js

Contient :
- Palette 9 couleurs avec ratios de contraste calculés
- Typographie (Caprasimo, DM Sans, DM Mono)
- Composants signatures (boutons primaire/secondaire, inputs, toasts, cards)
- Canards SVG (4 variantes de couleur, mariées, tailles 24/56/140px)
- Mode nuit pour Mare TV

### Welcome v2.html
**Statut** : validé pixel-correct
**Écran** : génération de pseudo + canard
**Vue cible** : `frontend/src/views/WelcomeView.vue`

Éléments clés :
- Header "The Pond." en grand Caprasimo
- Mini-pond teaser avec couple central
- Carte de génération avec canard 140px et contour Pond Deep
- Pseudo Caprasimo adaptatif sans truncation
- Bouton rerouler 56px circulaire centré
- Champ pseudo optionnel
- CTA primaire "PLONGER DANS L'ÉTANG" Duck Yellow 64px (à mettre à 
  jour en "PLONGER DANS LA MARE" lors de l'intégration)
- Zone install PWA discrète en bas

### Upload v2.html
**Statut** : validé pixel-correct
**Écran** : home action (upload photo/clip/vocal)
**Vue cible** : `frontend/src/views/UploadView.vue`

Éléments clés :
- Header minimal + compteur 42/52
- Salutation avec canard utilisateur ("Tes plumes scintillent magnifiquement")
- Grille asymétrique 1+2 :
  - BARBOTER (Duck Yellow plein largeur, 112px)
  - CANCANER (Cream Deep avec point rouge sur icône caméra) + 
    FAIRE COIN-COIN (Coral Soft) côte à côte 100px
- Sous-labels durées "15s" / "60s" en monospace 13px
- Variante A (avec carte défi active) et variante B (sans carte défi)

Note pour l'intégration :
- La carte défi doit être intégrée en bandeau au-dessus du bouton BARBOTER 
  plutôt que comme 4e CTA séparé (décision prise après les maquettes)

### Confirmation.html
**Statut** : validé avec ajustements à faire en code
**Écran** : confirmation post-upload (variante photo)
**Vue cible** : `frontend/src/views/ConfirmationView.vue`

Éléments clés :
- Bande dessinée 3 cases (frames 01, 02, 03) montrant canard qui plonge
- "Splash !" en grand Caprasimo Pond Deep avec ! en Duck Yellow
- Sous-message "Ton coin-coin a rejoint la mare"
- Card preview avec thumbnail
- CTA "CONTINUER À BARBOTER" Duck Yellow + liens secondaires

À corriger en code :
- Remplacer "ta photo · en ligne · à l'instant" par "Ta photo · barbote 
  dans la mare"
- Retirer "1.0 !" mystérieux dans frame 03
- Variantes clip et vocal à dériver en code (pas dans le HTML)

### Mosaic 4 screens.html
**Statut** : DRAFT (structures validées, détails à raffiner en code)
**Écrans** : galerie publique, clip player, vocal player, modération
**Vues cibles** :
- `frontend/src/views/GalleryView.vue` (variante simple — voir Mosaic 6 
  pour les 4 états réels)
- `frontend/src/views/ClipPlayerView.vue`
- `frontend/src/views/VoicePlayerView.vue`
- `frontend/src/views/OrchestrationView.vue` (anciennement modération, 
  à renommer)

Note importante :
- La galerie publique ici est une version SIMPLE qui ne respecte pas 
  le concept appareil photo jetable. La VRAIE galerie est dans 
  `Mosaic 6 screens.html` avec ses 4 états.
- La page "modération" doit être renommée en "orchestration" et adaptée 
  selon les specs de PROJECT.md (paliers configurables, pas de 
  modération de contenu).

### Mosaic 6 screens.html
**Statut** : DRAFT (structures validées, détails à raffiner en code)
**Écrans** : 4 états galerie + orchestration + Mare TV instructions
**Vues cibles** :
- `frontend/src/views/GalleryView.vue` (avec logique de 4 états)
- `frontend/src/views/OrchestrationView.vue` (version finale avec 
  paliers configurables)
- `frontend/src/views/MareTVInstructionsView.vue`

États de galerie représentés :
1. Mare endormie initiale (avant le premier palier visible)
2. Mare réveillée (un palier visible, prochain annoncé)
3. Mare nocturne (extinction de fin de soirée)
4. Canards confits (révélation totale du lendemain)

## Écrans manquants (non maquettés, à coder à partir des références)

Ces écrans doivent être développés en s'inspirant des conventions 
établies dans les écrans validés :

- **Slideshow projeté (Mare TV)** : variantes jour et nuit
  Vue cible : `frontend/src/views/MareTVView.vue`
  Référence : voir le design system v0.2 pour les patterns Mare TV
  
- **Capture in-app clip vidéo** : interface MediaRecorder plein écran 
  avec compte à rebours 15s
  Vue cible : `frontend/src/views/ClipCaptureView.vue`

- **Capture in-app vocal** : interface enregistrement avec waveform 
  en direct, timer 60s
  Vue cible : `frontend/src/views/VoiceCaptureView.vue`

- **Écran d'erreur d'upload** : canard triste, retry, message rassurant
  Vue cible : `frontend/src/views/ErrorView.vue`

- **Sheet d'instructions iOS install PWA** : drawer avec 3 étapes
  Composant : `frontend/src/components/InstallSheet.vue`

## Vocabulaire à appliquer en code

Plusieurs maquettes utilisent encore "l'étang" qu'il faut remplacer 
par "la mare" lors de l'intégration. Voir PROJECT.md section 
"Vocabulaire du projet" pour la liste complète.
