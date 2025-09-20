from typing import Literal

import arcade
from arcade import Texture
from arcade.types import XYWH
from loguru import logger

from pokemon import Monsters, Moves
from utils import POKEMON_SPRITES_PATH, save_state


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


class BaseMenu(arcade.View):

    def __init__(self, previous_scene: arcade.View, player_state: dict, selected_index):
        super().__init__()
        self.previous_scene = previous_scene
        self.player_state = player_state
        monster_name_list = {monster.name.lower() for monster in Monsters}
        self.team = self._collect_team()
        self.main_menu = None
        pokemon_images = {
            name.lower(): POKEMON_SPRITES_PATH / name.title() / "banner.png"
            for name in self.team.keys()
            if name.lower() in monster_name_list
        }
        self.loaded_textures = {
            name: arcade.load_texture(path) for name, path in pokemon_images.items()
        }
        self.index = len(self.team)
        self.selected_index = selected_index

    def _collect_team(self):
        return self.player_state.get("pokemons", {})


class Menu(arcade.View):

    def __init__(self, overworld_view: arcade.View, player_state: dict):
        super().__init__()
        self.overworld_view = overworld_view
        self.player_state = player_state
        self.pokdex = Pokedex(
            previous_scene=self.overworld_view, player_state=self.player_state
        )
        self.team_menu = TeamMenuView(
            previous_scene=self.overworld_view, player_state=self.player_state
        )
        self.team_menu.main_menu = self
        self.pokdex.main_menu = self
        self.menus = [
            self.pokdex,
            self.team_menu,
        ]
        self.current_menu_index = 0
        self.current_menu_view: BaseMenu = self.pokdex
        self.team = self.player_state.get("pokemons", {})

    def on_key_press(self, key, modifiers):
        """Controls to move between menus / go back to overworld. spicific controls are in the menu itself"""
        if key == arcade.key.ESCAPE:
            self.window.show_view(self.overworld_view)
        elif key == arcade.key.LCTRL:
            self.current_menu_index = (self.current_menu_index - 1) % len(self.menus)
            self.current_menu_view = self.menus[self.current_menu_index]
        elif key == arcade.key.LSHIFT:
            self.current_menu_index = (self.current_menu_index + 1) % len(self.menus)
            self.current_menu_view = self.menus[self.current_menu_index]
        else:
            self.current_menu_view.on_key_press(key, modifiers)

    def on_draw(self):
        self.clear()
        self.current_menu_view.on_draw()


class Pokedex(BaseMenu):

    def __init__(self, previous_scene: arcade.View, player_state: dict):
        super().__init__(previous_scene, player_state, selected_index=0)
        self.monster_name_list = {monster.name.lower() for monster in Monsters}
        self.pokedex_monsters = Monsters
        self.monster_list = []
        self.seen_names = [
            monster_member.lower() for monster_member in self.player_state["pokedex"]
        ]
        self.team_names = [monster_member for monster_member in self.team.keys()]

        self.pokemon_images = {
            team_monster.name.lower(): POKEMON_SPRITES_PATH
            / team_monster.name.title()
            / "banner.png"
            for team_monster in self.pokedex_monsters
            # if team_monster.name.lower() in self.seen_names
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

    def on_key_press(self, key, modifiers):
        length_menu = self.index
        if key == arcade.key.UP:
            self.selected_index = (self.selected_index - 1) % length_menu
        elif key == arcade.key.DOWN:
            self.selected_index = (self.selected_index + 1) % length_menu

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


class TeamMenuView(BaseMenu):

    def __init__(self, previous_scene: arcade.View, player_state: dict):
        super().__init__(previous_scene, player_state, selected_index=0)
        monster_name_list = {monster.name.lower() for monster in Monsters}
        pokemon_images = {
            name.lower(): POKEMON_SPRITES_PATH / name.title() / "banner.png"
            for name in self.team.keys()
            if name.lower() in monster_name_list
        }
        self.loaded_textures = {
            name: arcade.load_texture(path) for name, path in pokemon_images.items()
        }
        self.selected_monster = None

    def _collect_team(self):
        return self.player_state.get("pokemons", {})

    def _draw_list_of_mosters(self, x_start, y_start):
        for i, pokemon_name in enumerate(self.team):
            if i == self.selected_index:
                self.selected_monster = pokemon_name
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

    def on_key_press(self, key, modifiers):
        length_menu = self.index
        if key == arcade.key.UP:
            self.selected_index = (self.selected_index - 1) % length_menu
        elif key == arcade.key.DOWN:
            self.selected_index = (self.selected_index + 1) % length_menu
        elif key == arcade.key.SPACE:
            menu = MonsterMenu(self.main_menu, self.player_state, self.selected_index)
            self.window.show_view(menu)

    def draw_monster_info(self, x_start: int, width: int, y_start: int) -> None:
        PROFILE_MONSTER = 64 * 3
        try:
            data = self.team[self.selected_monster]
        except IndexError:
            raise ValueError("No pokemon is pressent in team! no info to see!")
        pokemon_name = self.selected_monster
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


def draw_moves_monster(
    moves: dict[str, str], start_y: int, start_x: int, title: str
) -> int:
    indent_space = 30
    arcade.Text(
        f"{title}:",
        start_x,
        start_y - 10,
        arcade.color.WHITE,
        font_size=16,
    ).draw()
    for j, move in enumerate(moves.values()):
        arcade.Text(
            f"- {move}",
            start_x + indent_space,
            start_y - 40 - j * 25,
            arcade.color.WHITE,
            font_size=12,
        ).draw()
    return start_y - 40 - j * 25


class MonsterMenu(BaseMenu):

    def __init__(self, previous_menu: Menu, player_state: dict, selected_index):
        super().__init__(previous_menu, player_state, selected_index)
        self.mode: Literal["browse", "assign"] = "browse"
        self.move_slot_index = 0
        self.index = 0
        self.location_drawings: dict[
            Literal["monster banner", "quest list", "description", "known moves"],
            tuple[int, int],
        ] = {
            "monster banner": (200, self.window.height - 200),
            "quest list": (self.window.width - 300, self.window.height - 200),
            "description": (350, self.window.height - 200),
            "known moves": (400, self.window.height / 2),
        }
        for i, monster in enumerate(self.team.keys()):
            if i == self.selected_index:
                self.selected = monster
                break

        self.pokemon_name = self.selected
        self.data = self.team[self.pokemon_name]
        self.monster = [
            monster for monster in Monsters if monster.name.lower() == self.pokemon_name
        ][0]

    def draw_monster_info(self) -> None:
        draw_monster_profile(
            pokemon_name=self.pokemon_name,
            center_x=self.location_drawings["monster banner"][0],
            center_y=self.location_drawings["monster banner"][1],
            textures=self.loaded_textures,
        )
        draw_monster_stats(
            stats_monster=self.data.get("stats", {}),
            y_start=self.location_drawings["description"][1],
            start_x=self.location_drawings["description"][0],
        )
        for j, (location, move) in enumerate(self.data.get("attacks", []).items()):
            color = arcade.color.WHITE
            if self.mode == "assign" and j == self.move_slot_index:
                color = arcade.color.YELLOW
            text_input = f"{location}: {move}" if move else f"{location}: not used"
            arcade.Text(
                text_input,
                self.location_drawings["known moves"][0],
                self.location_drawings["known moves"][1] - 40 - j * 25,
                color,
                font_size=12,
            ).draw()
        row_height = self.location_drawings["quest list"][1]
        colum_width = self.location_drawings["quest list"][0]
        arcade.Text(f"Move list of {self.monster.name}", colum_width, row_height)
        for i, move in enumerate(self.monster.moves):
            row_height = self.location_drawings["quest list"][1] - i * 40
            color = arcade.color.YELLOW if i == self.index else arcade.color.LIGHT_GRAY
            draw_quest_line(
                move,
                start_x=self.location_drawings["quest list"][0],
                start_y=row_height,
                color=color,
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
        self.draw_monster_info()
        arcade.Text(
            "ESC = Return  < | > = Navigate",
            self.window.width // 2,
            50,
            arcade.color.LIGHT_BLUE,
            font_size=14,
            anchor_x="center",
        ).draw()

    def on_key_press(self, key, modifiers):
        if self.mode == "browse":
            if key == arcade.key.ESCAPE:
                save_state(self.player_state)
                logger.debug(
                    f"attacks: {self.player_state['pokemons'][self.pokemon_name]['attacks']}"
                )
                self.window.show_view(self.previous_scene)
            elif key == arcade.key.DOWN:
                self.index = (self.index - 1) % len(self.monster.moves)
            elif key == arcade.key.UP:
                self.index = (self.index + 1) % len(self.monster.moves)
            elif key == arcade.key.SPACE:
                if self.monster.moves[self.index].quest_line.finised:
                    self.mode = "assign"
                    self.move_slot_index = 0
                else:
                    logger.debug("jammer man, quest nog niet complete")
        elif self.mode == "assign":
            if key == arcade.key.ESCAPE:
                self.mode = "browse"
            elif key == arcade.key.UP:
                self.move_slot_index = (self.move_slot_index - 1) % len(
                    self.data["attacks"]
                )
            elif key == arcade.key.DOWN:
                self.move_slot_index = (self.move_slot_index + 1) % len(
                    self.data["attacks"]
                )
            elif key == arcade.key.SPACE:
                chosen_move = self.monster.moves[self.index]
                key_attack = list(self.data["attacks"].keys())[self.move_slot_index]
                self.data["attacks"][key_attack] = chosen_move.name
                logger.debug(f"local:{self.selected}")
                logger.debug(f"player state:{self.player_state}")
                self.mode = "browse"


def draw_quest_line(
    move: Moves, start_y: int, start_x: int, color: arcade.color = arcade.color.WHITE
):
    arcade.Text(
        f"{move.name}",
        start_x,
        start_y,
        color,
        font_size=10,
    ).draw()
    if move.quest_line.finised:
        arcade.Text(
            f"{move.quest_line.quest} COMPLETED",
            start_x,
            start_y - 12,
            color,
            font_size=12,
        ).draw()
    else:
        arcade.Text(
            f"{move.quest_line.quest}: {move.quest_line.achieved_count}/{move.quest_line.objective_count}",
            start_x,
            start_y - 12,
            color,
            font_size=12,
        ).draw()
