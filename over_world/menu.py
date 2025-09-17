from typing import Literal

import arcade
from arcade import Texture
from arcade.types import XYWH
from loguru import logger

from pokemon import Monsters, Moves
from utils import POKEMON_SPRITES_PATH


def draw_monster_profile(
    pokemon_name: str, center_x, center_y, textures: dict[str, Texture]
):
    if not pokemon_name in textures:
        raise ValueError("pokemon not found for profile")
    tex = textures[pokemon_name]
    width = tex.width * 3  # TODO: remove *3 when bigger profile picture is made
    height = tex.height * 3  # TODO: remove *3 when bigger profile picture is made
    rect = XYWH(center_x, center_y, width, height)
    arcade.draw_texture_rect(
        texture=tex,
        rect=rect,
        color=arcade.color.WHITE,
        angle=0,
        alpha=255,
        pixelated=False,
    )
    return center_y - height / 2


class Menu(arcade.View):

    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.pokdex = Pokedex(
            overworld_view=self.overworld_view, player_state=self.player_state
        )
        self.team_menu = TeamMenuView(
            overworld_view=self.overworld_view, player_state=self.player_state
        )
        self.monster_info = MonsterMenu(
            overworld_view=self.overworld_view,
            player_state=self.player_state,
            selected_index=0,
        )
        self.menu = [
            self.pokdex,
            self.team_menu,
        ]
        self.current_menu = 0
        self.team = self.player_state.get("pokemons", {})
        self.view_mode: Literal["menu", "monster_info"] = "menu"

    def on_key_press(self, key, modifiers):
        menu_selected = self.menu[self.current_menu]
        length_menu = menu_selected.index
        logger.debug(f"length list menu:{length_menu}")
        if key == arcade.key.ESCAPE:
            if self.view_mode == "menu":
                self.window.show_view(self.overworld_view)
            elif self.view_mode == "monster_info":
                self.view_mode = "menu"
        if key == arcade.key.SPACE:
            if menu_selected == self.team_menu:
                self.monster_info.index = menu_selected.index
                self.monster_info.selected_index = menu_selected.selected_index
                self.view_mode = "monster_info"

        elif key == arcade.key.UP:
            menu_selected.selected_index = (
                menu_selected.selected_index - 1
            ) % length_menu
            logger.debug(f"index=:{menu_selected.selected_index}")

        elif key == arcade.key.DOWN:
            menu_selected.selected_index = (
                menu_selected.selected_index + 1
            ) % length_menu
            logger.debug(f"index=:{menu_selected.selected_index}")

        elif key == arcade.key.RIGHT:
            self.current_menu = (self.current_menu - 1) % len(self.menu)

        elif key == arcade.key.LEFT:
            self.current_menu = (self.current_menu + 1) % len(self.menu)

    def on_draw(self):
        self.clear()
        if self.view_mode == "menu":
            self.menu[self.current_menu].on_draw()
        elif self.view_mode == "monster_info":
            self.monster_info.on_draw()


class Pokedex(arcade.View):
    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.monster_name_list = {monster.name.lower() for monster in Monsters}
        self.team = self._collect_team()
        monster_name_list = [monster.name.lower() for monster in Monsters]
        self.index = len(monster_name_list)
        self.selected_index = 0
        self.pokedex_monsters = Monsters
        self.monster_list = []
        self.seen_names = [
            monster_member.lower() for monster_member in self.player_state["pokedex"]
        ]
        self.team_names = [
            list(monster_member.keys())[0].lower() for monster_member in self.team
        ]

        self.pokemon_images = {
            team_monster.name.lower(): POKEMON_SPRITES_PATH
            / team_monster.name.title()
            / "banner.png"
            for team_monster in self.pokedex_monsters
            if team_monster.name.lower() in self.seen_names
        }
        self.loaded_textures = {
            name: arcade.load_texture(path)
            for name, path in self.pokemon_images.items()
        }

    def _collect_team(self):
        return self.player_state.get("pokemons", {})

    def _draw_list_of_mosters(
        self,
        x_start,
        y_start,
    ):
        for i, pokemon in enumerate(self.pokedex_monsters):
            pokemon_name = pokemon.name.lower()
            y = y_start - i * 50
            color = (
                arcade.color.YELLOW
                if i == self.selected_index
                else arcade.color.LIGHT_GRAY
            )
            if pokemon_name in self.team_names or pokemon_name in self.seen_names:
                self.monster_list.append(pokemon_name)
                arcade.Text(
                    f"{i}:\t{pokemon_name}",
                    x_start,
                    y,
                    color,
                    font_size=18,
                ).draw()
            else:
                self.monster_list.append("????")
                arcade.Text(
                    f"{i}:\t????",
                    x_start,
                    y,
                    color,
                    font_size=18,
                ).draw()

    def draw_monster_info(self, x_start: int, width: int, y_start: int) -> None:
        profile_monster = 64 * 3
        try:
            selected = self.pokedex_monsters[self.selected_index]
        except IndexError:
            raise ValueError("No pokemon is pressent in team! no info to see!")
        pokemon_name = selected.name.lower()
        draw_monster_profile(
            pokemon_name=pokemon_name,
            center_x=x_start + width - profile_monster / 2,
            center_y=y_start - profile_monster / 2,
            textures=self.loaded_textures,
        )
        if pokemon_name in self.team_names:
            texts = [
                f"monster seen in route: {selected.fields}",
                f"description:{selected.description.description}",
                f"type:{selected.description.type}",
            ]
            for i, text in enumerate(texts):
                arcade.Text(
                    text,
                    x_start + 20,
                    y_start - profile_monster + 20 * i,
                    arcade.color.WHITE,
                    font_size=16,
                ).draw()
        elif pokemon_name in self.seen_names:
            arcade.Text(
                f"more info at route:{selected.fields}",
                x_start + 20,
                y_start - profile_monster,
                arcade.color.WHITE,
                font_size=16,
                anchor_x="center",
            ).draw()

    def on_draw(self):
        self.clear()
        arcade.Text(
            "Monster Guide",
            self.window.width // 2,
            self.window.height - 50,
            arcade.color.WHITE,
            font_size=24,
            anchor_x="center",
        ).draw()
        self._draw_list_of_mosters(100, self.window.height - 200)
        if self.monster_list[self.selected_index] != "????":
            self.draw_monster_info(
                x_start=self.window.width // 2,
                width=self.window.width // 2,
                y_start=self.window.height - 200,
            )
        arcade.Text(
            "ESC = Return  < | > = Navigate",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            font_size=14,
            anchor_x="center",
        ).draw()


class TeamMenuView(arcade.View):
    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.team = self._collect_team()
        self.index = len(self.team)
        self.selected_index = 0
        monster_name_list = {monster.name.lower() for monster in Monsters}

        pokemon_images = {
            name.lower(): POKEMON_SPRITES_PATH / name.title() / "banner.png"
            for team_monster in self.team
            for (name,) in [team_monster.keys()]
            if name.lower() in monster_name_list
        }
        self.loaded_textures = {
            name: arcade.load_texture(path) for name, path in pokemon_images.items()
        }

    def _collect_team(self):
        return self.player_state.get("pokemons", {})

    def _draw_list_of_mosters(self, x_start, y_start):
        for i, pokemon in enumerate(self.team):
            pokemon_name = list(pokemon.keys())[0]
            y = y_start - i * 50
            color = (
                arcade.color.YELLOW
                if i == self.selected_index
                else arcade.color.LIGHT_GRAY
            )
            arcade.Text(
                pokemon_name,
                x_start,
                y,
                color,
                font_size=18,
            ).draw()

    def draw_monster_info(self, x_start: int, width: int, y_start: int) -> None:
        PROFILE_MONSTER = 64 * 3
        try:
            selected = self.team[self.selected_index]
        except IndexError:
            raise ValueError("No pokemon is pressent in team! no info to see!")
        pokemon_name = next(iter(selected))
        data = selected[pokemon_name]
        draw_monster_profile(
            pokemon_name=pokemon_name,
            center_x=x_start + width - PROFILE_MONSTER / 2,
            center_y=y_start - PROFILE_MONSTER / 2,
            textures=self.loaded_textures,
        )
        end_y = draw_monster_stats(
            stats_monster=data.get("stats", {}), y_start=y_start, start_x=x_start
        )
        draw_moves_monster(
            moves=data.get("attacks", []), start_y=end_y, start_x=x_start, title="Moves"
        )

    def on_draw(self):
        self.clear()
        arcade.Text(
            "Your Pokémon Team",
            self.window.width // 2,
            self.window.height - 50,
            arcade.color.WHITE,
            font_size=24,
            anchor_x="center",
        ).draw()
        self._draw_list_of_mosters(100, self.window.height - 200)
        self.draw_monster_info(
            x_start=self.window.width // 2,
            width=self.window.width // 2,
            y_start=self.window.height - 200,
        )

        arcade.Text(
            "ESC = Return  < | > = Navigate",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            font_size=14,
            anchor_x="center",
        ).draw()


def draw_monster_stats(
    stats_monster: dict[str, dict[str, str]], y_start: int, start_x: int
) -> int:
    for stat_name, value in stats_monster.items():
        stats = arcade.Text(
            f"{stat_name.capitalize()}: {value}",
            start_x,
            y_start,
            arcade.color.WHITE,
            font_size=16,
        )
        stats.draw()
        y_start -= 30
    return y_start


def draw_moves_monster(moves: list[str], start_y: int, start_x: int, title: str) -> int:
    indent_space = 30
    arcade.Text(
        f"{title}:",
        start_x,
        start_y - 10,
        arcade.color.WHITE,
        font_size=16,
    ).draw()
    for j, move in enumerate(moves):
        arcade.Text(
            f"- {move}",
            start_x + indent_space,
            start_y - 40 - j * 25,
            arcade.color.WHITE,
            font_size=12,
        ).draw()
    return start_y - 40 - j * 25


class MonsterMenu(arcade.View):
    def __init__(self, overworld_view: arcade.View, player_state: dict, selected_index):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        monster_name_list = {monster.name.lower() for monster in Monsters}
        self.team = self._collect_team()
        pokemon_images = {
            name.lower(): POKEMON_SPRITES_PATH / name.title() / "banner.png"
            for team_monster in self.team
            for (name,) in [team_monster.keys()]
            if name.lower() in monster_name_list
        }
        self.loaded_textures = {
            name: arcade.load_texture(path) for name, path in pokemon_images.items()
        }
        self.index = len(self.team)
        self.selected_index = selected_index
        self.location_drawings: dict[
            Literal["monster banner", "quest list", "description", "known moves"],
            tuple[int, int],
        ] = {
            "monster banner": (200, self.window.height - 200),
            "quest list": (self.window.width - 300, self.window.height - 200),
            "description": (350, self.window.height - 200),
            "known moves": (400, self.window.height / 2),
        }

    def _collect_team(self):
        return self.player_state.get("pokemons", {})

    def draw_monster_info(self, x_start: int, width: int, y_start: int) -> None:
        try:
            selected = self.team[self.selected_index]
        except IndexError:
            raise ValueError("No pokemon is pressent in team! no info to see!")
        pokemon_name = next(iter(selected))
        monster = [
            monster for monster in Monsters if monster.name.lower() == pokemon_name
        ][0]
        data = selected[pokemon_name]
        draw_monster_profile(
            pokemon_name=pokemon_name,
            center_x=self.location_drawings["monster banner"][0],
            center_y=self.location_drawings["monster banner"][1],
            textures=self.loaded_textures,
        )
        draw_monster_stats(
            stats_monster=data.get("stats", {}),
            y_start=self.location_drawings["description"][1],
            start_x=self.location_drawings["description"][0],
        )
        draw_moves_monster(
            moves=data.get("attacks", []),
            start_y=self.location_drawings["known moves"][1],
            start_x=self.location_drawings["known moves"][0],
            title="Moves",
        )
        row_height = self.location_drawings["quest list"][1]
        colum_width = self.location_drawings["quest list"][0]
        arcade.Text(f"Move list of {monster.name}", colum_width, row_height)
        for move in monster.moves:
            row_height -= 40
            draw_quest_line(
                move=move,
                start_x=colum_width,
                start_y=row_height,
            )

    def on_draw(self):
        self.clear()
        arcade.Text(
            "Your Pokémon Team",
            self.window.width // 2,
            self.window.height - 50,
            arcade.color.WHITE,
            font_size=24,
            anchor_x="center",
        ).draw()
        self.draw_monster_info(
            x_start=self.window.width // 2,
            width=self.window.width // 2,
            y_start=self.window.height - 200,
        )

        arcade.Text(
            "ESC = Return  < | > = Navigate",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            font_size=14,
            anchor_x="center",
        ).draw()


def draw_quest_line(move: Moves, start_y: int, start_x: int):
    arcade.Text(
        f"{move.name}",
        start_x,
        start_y,
        arcade.color.WHITE,
        font_size=10,
    ).draw()
    if move.quest_line.finised:
        arcade.Text(
            f"{move.quest_line.quest} COMPLETED",
            start_x,
            start_y - 12,
            arcade.color.WHITE,
            font_size=12,
        ).draw()
    else:
        arcade.Text(
            f"{move.quest_line.quest}: {move.quest_line.achieved_count}/{move.quest_line.objective_count}",
            start_x,
            start_y - 12,
            arcade.color.WHITE,
            font_size=12,
        ).draw()
