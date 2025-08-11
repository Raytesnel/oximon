from pathlib import Path

import arcade
from arcade import load_tilemap
from loguru import logger

from houses.helper_stuff import check_object_collision
from lifeforms.characters import Player
from utils import ASSETS_PATH


class BaseMap(arcade.View):
    def __init__(self,player_location_key:str,map:Path,possible_gates:list[str]):
        super().__init__()
        self.tile_map = load_tilemap(
            map,
            scaling=2.0,
            use_spatial_hash=True,
        )
        self.possible_gates = possible_gates
        self.player_location_key  = player_location_key
        self.camera = arcade.Camera2D()
        self.player_list = arcade.SpriteList()
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.player = Player(ASSETS_PATH/ "sprites/player")
        self.player_list.append(self.player)
        self.walls = self.scene["abandoned"]
        self.y_sorted_sprites = arcade.SpriteList()
        self.possible_gate_objects = [
                        o
                        for o in self.tile_map.object_lists["objects"]
                        if o.name == self.possible_gates
                    ]
        self.setup()

    def setup(self):
        try:
            start = next((o for o in self.tile_map.object_lists["objects"] if o.name == self.player_location_key), None)
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


    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.player_list.draw()
        self.y_sorted_sprites.draw()
        self.scene.get_sprite_list("voorgrond").draw()

    def on_update(self, delta_time):
        self.scene.update_animation(delta_time)
        for sprite in self.y_sorted_sprites:
            if hasattr(sprite, "_hit_box") and sprite._hit_box:
                min_hitbox_y = min(point[1] for point in sprite._hit_box.points)
                sprite.depth_y = sprite.center_y + min_hitbox_y
            else:
                sprite.depth_y = sprite.center_y
        self.y_sorted_sprites.sort(key=lambda s: -getattr(s, "depth_y", s.center_y))

        for gate in self.possible_gates:
            try:
                gate_object = next(
                    (
                        o
                        for o in self.tile_map.object_lists["objects"]
                        if o.name == gate
                    ),
                    None,
                )
            except KeyError:
                raise KeyError("house not found")
            if check_object_collision(self.player, gate_object):
                logger.debug("going in the house.")
                from houses.MapConfig import MapConfigs
                mapchanger = MapConfigs()
                view = mapchanger.load_map_by_id(mapchanger.get_shizzle(gate_object),gate_object.name)
                self.window.clear()
                self.window.show_view(view)
        self.player_list.update()
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y
        else:
            self.player_list.update_animation(delta_time)
        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)

        # TODO player grass is to small due to hitbox (probly a new special varibel needed instead of _hit_box

    def _read_all_entrances(self,):
        pass

    def on_key_press(self, key, modifiers):
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

    def on_key_release(self, key, modifiers):
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0
