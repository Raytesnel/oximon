import arcade
from arcade import TileMap, View, load_spritesheet
from loguru import logger

from houses.npc import NPC
from lifeforms.characters import Player
from utils import ASSETS_PATH


class HouseMap(arcade.View):
    def __init__(self, player:Player, overworld_map:View,tile_map:TileMap):
        super().__init__()
        self.over_world = overworld_map
        self.player_list = arcade.SpriteList()
        self.npc_list = arcade.SpriteList()

        self.player = player
        self.player_list.append(self.player )
        self.all_sprites = []
        self.tile_map = tile_map
        self.scene=None
        self.walls =None
        self.camera = arcade.Camera2D()
        self.npc = NPC(sheet_path=ASSETS_PATH/"sprites"/"npcs"/"SpriteSheet.png")
        self.npc.scale = 2.0
        self.npc_list.append(self.npc)
        self.exit = None
        self.setup()

    def setup(self):
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.walls = self.scene["abandoned"]
        try:
            start = next((o for o in self.tile_map.object_lists["objects"] if o.name == "player-start"), None)
            logger.debug("found player start")
        except KeyError:
            raise ValueError(" player start not found")
        self.player.center_x = start.shape[0][0]
        self.player.center_y = start.shape[0][1]
        try:
            npc_start = next((o for o in self.tile_map.object_lists["objects"] if o.name == "npc_dude"), None)
            logger.debug("found npc start")
        except KeyError:
            raise ValueError(" player npc not found")
        self.npc.position = npc_start.shape[0]
        try:
            self.exit = next((o for o in self.tile_map.object_lists["objects"] if o.name == "exit"), None)
            logger.debug("found npc start")
        except KeyError:
            raise ValueError(" exit of house not found")

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.scene["achtergrond"].draw()
        self.walls.draw()
        self.player_list.draw()
        self.npc_list.draw()
        self.scene["voorgrond"].draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.scene.update_animation(delta_time)
        self.player_list.update_animation(delta_time)
        self.npc_list.update()
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y
        if arcade.check_for_collision_with_list(self.npc, self.walls):
            self.npc.center_x -= self.npc.change_x
            self.npc.center_y -= self.npc.change_y
        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)
        if check_object_collision(self.player,self.exit):
            self.window.show_view(self.over_world)

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
def check_object_collision(player, obj):
    (left, top), (right, _), (_, _), (_, bottom) = obj.shape
    return (
        player.right > left and
        player.left < right and
        player.top > bottom and
        player.bottom < top
    )
