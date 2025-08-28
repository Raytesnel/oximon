from pathlib import Path

import arcade
import yaml
from arcade import load_tilemap
from loguru import logger

from houses.helper_stuff import check_object_collision
from lifeforms.characters import Player
from over_world.menu import TeamMenuView
from utils import ASSETS_PATH, SCREEN_WIDTH, save_state, load_state


class BaseMap(arcade.View):
    def __init__(self, player_location_key: str, map: Path, possible_gates: list[str]):
        super().__init__()
        self.tile_map = load_tilemap(
            map,
            scaling=2.0,
            use_spatial_hash=True,
        )
        self.npc_list = arcade.SpriteList()
        self.possible_gates = possible_gates
        self.player_location_key = player_location_key
        self.camera = arcade.Camera2D()
        self.gui_camera = arcade.Camera2D()
        self.dialog: DialogPanel | None = None
        self.player_list = arcade.SpriteList()
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.player = Player(ASSETS_PATH / "sprites/player")
        self.player_list.append(self.player)
        self.walls = self.scene["abandoned"]
        self.y_sorted_sprites = arcade.SpriteList()
        self.possible_gate_objects = None
        self.player_state = load_state()
        self.setup()

    def setup(self):
        try:
            start = next(
                (
                    o
                    for o in self.tile_map.object_lists["objects"]
                    if o.name == self.player_location_key
                ),
                None,
            )
            print("found player start")
        except KeyError:
            print(" player start not found")
            start = None
        if start:
            self.player.center_x = start.shape[0]
            self.player.center_y = start.shape[1]
            logger.debug("player start set")
        else:
            self.player.center_x = 400
            self.player.center_y = 400
            logger.debug("player start set on default")
        self.y_sorted_sprites.append(self.player)
        for gate in self.possible_gates:

            self.possible_gate_objects = [
                o for o in self.tile_map.object_lists["objects"] if o.name == gate
            ]

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.player_list.draw()
        self.y_sorted_sprites.draw()
        self.scene.get_sprite_list("voorgrond").draw()
        self.gui_camera.use()
        if self.dialog and self.dialog.active:
            self.dialog.on_draw()

    def on_update(self, delta_time):
        if self.dialog and self.dialog.active:
            return
        self.scene.update_animation(delta_time)
        for sprite in self.y_sorted_sprites:
            if hasattr(sprite, "_hit_box") and sprite._hit_box:
                min_hitbox_y = min(point[1] for point in sprite._hit_box.points)
                sprite.depth_y = sprite.center_y + min_hitbox_y
            else:
                sprite.depth_y = sprite.center_y
        self.y_sorted_sprites.sort(key=lambda s: -getattr(s, "depth_y", s.center_y))

        for gate in self.possible_gate_objects:
            if check_object_collision(self.player, gate):
                logger.debug("going in the house.")
                from houses.MapConfig import MapConfigs

                mapchanger = MapConfigs()
                view = mapchanger.load_map_by_id(
                    mapchanger.get_shizzle(gate), gate.name
                )
                self.window.clear()
                self.window.show_view(view)
        self.player_list.update()
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y
        else:
            self.player_list.update_animation(delta_time)
        # Calculate world size in pixels
        map_width = (
            self.tile_map.width * self.tile_map.tile_width * self.tile_map.scaling
        )
        map_height = (
            self.tile_map.height * self.tile_map.tile_height * self.tile_map.scaling
        )

        screen_width, screen_height = self.window.width, self.window.height

        # Clamp camera position so it doesn't scroll outside map boundaries
        cam_x = max(
            screen_width // 2, min(self.player.center_x, map_width - screen_width // 2)
        )
        cam_y = max(
            screen_height // 2,
            min(self.player.center_y, map_height - screen_height // 2),
        )

        self.camera.position = arcade.Vec2(cam_x, cam_y)

    def _read_all_entrances(
        self,
    ):
        pass

    def on_key_press(self, key, modifiers):
        if key == arcade.key.SPACE:
            for npc_sprite in self.npc_list:
                if arcade.check_for_collision(self.player, npc_sprite):
                    if not self.dialog or not self.dialog.active:
                        self.dialog = DialogPanel(npc_sprite.name, self.player_state)
                        return

        if self.dialog and self.dialog.active:
            self.dialog.on_key_press(key, modifiers)
            return

        self.player.change_x = 0
        self.player.change_y = 0
        if key == arcade.key.UP:
            self.player.change_y = 3
        elif key == arcade.key.DOWN:
            self.player.change_y = -3
        elif key == arcade.key.LEFT:
            self.player.change_x = -3
        elif key == arcade.key.RIGHT:
            self.player.change_x = 3
        elif key == arcade.key.ESCAPE:
            menu = TeamMenuView(self, self.player_state)
            self.window.show_view(menu)

    def on_key_release(self, key, modifiers):
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0


class DialogPanel(arcade.Section):
    def __init__(
        self,
        npc_id: str,
        player_state: dict,
    ):
        super().__init__(0, 0, SCREEN_WIDTH, 120, enabled=True)
        self.npc_id = npc_id
        self.player_state = player_state
        self.current_line = 0
        self.active = True
        self.choices_active = False
        self.selected_choice = 0
        self.text_color = arcade.color.WHITE
        self.download_monster = False
        self.kill_monster = False
        self._read_dialog_file()
        try:
            self.current_node = self.get_available_node()
        except Exception:
            logger.error(
                "tacle this problem!, we have somthing when we have no available lines."
            )
        self.lines = self.current_node["lines"]
        self.text_obj = arcade.Text(
            text=self.lines[0]["text"],
            x=20,
            y=self.height // 2 - 20,
            color=self.text_color,
            font_size=16,
            width=self.width - 40,
        )

    def _read_dialog_file(self):
        dialog_file = ASSETS_PATH / "map" / "dialog.yaml"
        with open(dialog_file, "r", encoding="utf-8") as f:
            self.dialog_data = yaml.safe_load(f)

    def get_available_node(self):
        npc_data = self.dialog_data[self.npc_id]["dialogs"]
        for name, n in npc_data.items():
            conds = n.get("conditions", {})
            if all(
                self.player_state.get("quests").get(k) == v for k, v in conds.items()
            ):
                return n

    def advance_line(self):
        self.current_line += 1
        if self.current_line >= len(self.lines):
            # Node finished
            self.check_next()
        else:
            line = self.lines[self.current_line]
            self.text_obj.text = line["text"]
            self.text_obj.color = self.text_color

    def check_next(self):
        if "quest_update" in self.current_node:
            for key, val in self.current_node["quest_update"].items():
                self.player_state["quests"][key] = val
            save_state(self.player_state)
            self.player_state = load_state()
        if "pokemons" in self.current_node:
            for pokemon in self.current_node["pokemons"]:
                self.player_state["pokemons"].append(pokemon)
            save_state(self.player_state)

        # Handle choices
        if "choices" in self.current_node:
            self.choices_active = True
            self.selected_choice = 0
        # Auto-next node
        elif "next" in self.current_node:
            next_node_name = self.current_node["next"]
            self.current_node = self.dialog_data[self.npc_id]["dialogs"][next_node_name]
            self.lines = self.current_node["lines"]
            self.current_line = 0
            line = self.lines[0]
            self.text_obj.text = line["text"]
            self.text_obj.color = self.text_color
        else:
            self.active = False  # Close dialog

    def on_draw(self):
        if not self.active:
            return
        arcade.draw_lbwh_rectangle_filled(
            bottom=0,
            left=0,
            width=self.width,
            height=self.height,
            color=arcade.color.REDWOOD,
        )
        if not self.choices_active:
            self.text_obj.draw()
        else:
            for i, choice in enumerate(self.current_node["choices"]):
                color = (
                    arcade.color.YELLOW
                    if i == self.selected_choice
                    else arcade.color.WHITE
                )
                choice = arcade.Text(
                    choice["text"],
                    20,
                    80 - i * 30,
                    color,
                    font_size=16,
                )
                choice.draw()

    def on_key_press(self, key, modifiers):
        if not self.active:
            return
        if self.choices_active:
            if key == arcade.key.UP:
                self.selected_choice = max(0, self.selected_choice - 1)
            elif key == arcade.key.DOWN:
                self.selected_choice = min(
                    len(self.current_node["choices"]) - 1, self.selected_choice + 1
                )
            elif key == arcade.key.SPACE:
                chosen = self.current_node["choices"][self.selected_choice]
                next_node_name = chosen["next"]
                if next_node_name == "download_monster":
                    self.download_monster = True
                elif next_node_name == "kill_monster":
                    logger.debug("pokemon needs to be removed.")
                    self.kill_monster = True
                self.current_node = self.dialog_data[self.npc_id]["dialogs"][
                    next_node_name
                ]
                self.lines = self.current_node["lines"]
                self.current_line = 0
                line = self.lines[0]
                self.text_obj.text = line["text"]
                self.text_obj.color = self.text_color
                self.choices_active = False
        else:
            if key == arcade.key.SPACE:
                self.advance_line()
