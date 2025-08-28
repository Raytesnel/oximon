import arcade
from arcade.types import XYWH

from utils import POKEMON_SPRITES_PATH


class TeamMenuView(arcade.View):
    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.team = self._collect_team()
        self.selected_index = 0  # welke Pokémon is gekozen
        self.pokemon_images = {
            "charmander": POKEMON_SPRITES_PATH / "Charmander" / "banner.png",
            "bulbasaur": POKEMON_SPRITES_PATH / "Bulbasaur" / "banner.png",
        }
        self.loaded_textures = {
            name: arcade.load_texture(path)
            for name, path in self.pokemon_images.items()
        }

    def _collect_team(self):
        return self.player_state.get("pokemons", [])

    def on_draw(self):
        self.clear()
        # Titel
        title = arcade.Text(
            "Your Pokémon Team",
            self.window.width // 2,
            self.window.height - 50,
            arcade.color.WHITE,
            font_size=24,
            anchor_x="center",
        )
        title.draw()

        # ---- LINKS: lijst van teamleden ----
        start_y = self.window.height - 150
        for i, pokemon in enumerate(self.team):
            pokemon_name = list(pokemon.keys())[0]
            y = start_y - i * 50
            color = (
                arcade.color.YELLOW
                if i == self.selected_index
                else arcade.color.LIGHT_GRAY
            )
            pokemon = arcade.Text(
                pokemon_name,
                100,
                y,
                color,
                font_size=18,
            )
            pokemon.draw()

        # ---- RECHTS: details van geselecteerde Pokémon ----
        if self.team:
            selected = self.team[self.selected_index]
            pokemon_name = list(selected.keys())[0]
            data = selected[pokemon_name]

            # Portrait
            if pokemon_name in self.loaded_textures:
                tex = self.loaded_textures[pokemon_name]
                scale = 3.0
                width = tex.width * scale
                height = tex.height * scale

                # Positie: rechts, gecentreerd
                center_x = self.window.width - width / 2 - 50
                center_y = self.window.height - height / 2 - 100

                rect = XYWH(center_x, center_y, width, height)

                arcade.draw_texture_rect(
                    texture=tex,
                    rect=rect,
                    color=arcade.color.WHITE,
                    angle=0,
                    alpha=255,
                    pixelated=False,
                )
            # Stats
            stats = data.get("stats", {})
            y_stats = self.window.height - 200
            for stat_name, value in stats.items():
                stats = arcade.Text(
                    f"{stat_name.capitalize()}: {value}",
                    self.window.width - 350,
                    y_stats,
                    arcade.color.WHITE,
                    font_size=16,
                )
                stats.draw()
                y_stats -= 30

            # Moves
            moves = data.get("attacks", [])
            move_text = arcade.Text(
                "Moves:",
                self.window.width - 350,
                y_stats - 10,
                arcade.color.WHITE,
                font_size=16,
            )
            move_text.draw()
            for j, move in enumerate(moves):
                moves = arcade.Text(
                    f"- {move}",
                    self.window.width - 320,
                    y_stats - 40 - j * 25,
                    arcade.color.WHITE,
                    font_size=12,
                )
                moves.draw()

        # Hint
        signs = arcade.Text(
            "ESC = Return  < | > = Navigate",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            font_size=14,
            anchor_x="center",
        )
        signs.draw()

    def on_key_press(self, key, modifiers):
        if key == arcade.key.ESCAPE:
            self.window.show_view(self.overworld_view)

        elif key == arcade.key.UP:
            self.selected_index = (self.selected_index - 1) % len(self.team)

        elif key == arcade.key.DOWN:
            self.selected_index = (self.selected_index + 1) % len(self.team)
