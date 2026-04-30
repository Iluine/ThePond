/**
 * Type unique partagé pour la couleur d'un canard. Référence dans :
 *   - components/Duck.vue (props.color)
 *   - components/Pond.vue (PondDuck.color)
 *   - stores/user.ts (Canard.duck_color)
 *   - types/snapshot.ts (Media.user_color)
 *   - migrations/0001_init.sql CHECK (duck_color IN ('yellow',...))
 *
 * Si on ajoute une couleur, mettre à jour la check constraint SQLite
 * et les variantes dans Duck.vue.
 */
export type DuckColor = 'yellow' | 'white' | 'blue' | 'rainbow'
