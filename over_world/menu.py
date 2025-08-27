import arcade


class TeamMenuView(arcade.View):
    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.team = self._collect_team()

    def _collect_team(self):
        return self.player_state.get("pokemons")

    def on_draw(self):
        self.clear()
        arcade.draw_text(
            "Your Pokémon Team",
            self.window.width // 2,
            self.window.height - 50,
            arcade.color.WHITE,
            font_size=24,
            anchor_x="center",
        )

        for i, pokemon in enumerate(self.team):
            y = self.window.height - 150 - i * 100
            pokemon_name = list(pokemon.keys())[0]
            pokemon = pokemon[pokemon_name]
            text = f"{pokemon_name} - HP: {pokemon['stats']['health']} - Attack: {pokemon['stats']['attack']}"
            arcade.draw_text(text, 100, y, arcade.color.YELLOW, 16)
            moves = ", ".join(pokemon["attacks"])
            arcade.draw_text(
                f"Moves: {moves}", 120, y - 30, arcade.color.LIGHT_GRAY, 14
            )

        arcade.draw_text(
            "Press ESC to return",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            14,
            anchor_x="center",
        )

    def on_key_press(self, key, modifiers):
        if key == arcade.key.ESCAPE:
            # Return to overworld
            self.window.show_view(self.overworld_view)
