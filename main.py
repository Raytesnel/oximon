import os
import random

import arcade
from arcade import load_tilemap
from loguru import logger

from houses.MapLoader import HouseMap
from initate_battle import BattleSplashView
from lifeforms.characters import Player
from lifeforms.pokemons import WildPokemon
from pokemon import Pokemons

SCREEN_WIDTH = 800
SCREEN_HEIGHT = 600
SCREEN_TITLE = "PokeSmash"

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")


class OverworldView(arcade.View):
    def __init__(self):
        super().__init__()
        self.count = False
        self.max_pokemon_per_bush = 3 # TODO: calculate this with surface area of the field.
        self.counter_pokemon =0
        self.pokemon_fields = {}
        self.all_sprites = []
        self.hous_graveyard_1 =None

    def setup(self):
        self.player_list = arcade.SpriteList()
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.camera = arcade.Camera2D()
        self.tile_map = load_tilemap(
            os.path.join(ASSETS_PATH, "map/graveyard.tmx"),
            scaling=2.0,
            use_spatial_hash=True,
        )
        self.scene = arcade.Scene.from_tilemap(self.tile_map)

        try:
            start = next((o for o in self.tile_map.object_lists["objects"] if o.name == "player-start"), None)
            print("found player start")
        except KeyError:
            print(" player start not found")
            start = None
        self.player = Player(asset_path)
        if start:
            self.player.center_x = start.shape[0][0]
            self.player.center_y = start.shape[0][1]
        else:
            self.player.center_x = SCREEN_WIDTH // 2
            self.player.center_y = SCREEN_HEIGHT // 2
        self.player.scale = 1.0
        self.player_list.append(self.player)
        self.walls = self.scene["abandoned"]
        self.wild_pokemon_list = arcade.SpriteList()
        self.y_sorted_sprites = arcade.SpriteList()
        self.y_sorted_sprites.extend(self.scene["grass"])
        self.y_sorted_sprites.extend(self.wild_pokemon_list)
        self.y_sorted_sprites.append(self.player)
        try:
            self.hous_graveyard_1 = next((o for o in self.tile_map.object_lists["objects"] if o.name == "graveyard-house"), None)
        except KeyError:
            raise KeyError("house not found")

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.y_sorted_sprites.draw()
        self.scene.get_sprite_list("voorgrond").draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.scene.update_animation(delta_time)
        self.player_list.update_animation(delta_time)
        if check_object_collision(self.player,self.hous_graveyard_1):
            logger.debug("going in the house.")
            house = HouseMap(
                player = self.player,
                overworld_map = self,
                tile_map=load_tilemap(
            os.path.join(ASSETS_PATH, "map/inside_house.tmx"),
            scaling=2.0,
            use_spatial_hash=True,
        )
            )
            self.window.show_view(house)
            pass  # load house
        self.wild_pokemon_list.update()
        pokemon_fields = self.tile_map.object_lists["pokemonFields"]
        pokemon_field = random.choice(pokemon_fields)
        if pokemon_field.name not in self.pokemon_fields.keys():
            self.pokemon_fields[pokemon_field.name] = []
        pokemons_in_field =[pokemon for pokemon in self.wild_pokemon_list if pokemon_field.name == pokemon.field]
        if self.max_pokemon_per_bush > len(pokemons_in_field):
            if self.counter_pokemon > random.uniform(0.5,1):
                left_top, right_top, right_bottom, left_bottom= pokemon_field.shape
                bounds = (left_top[0], right_top[0], left_bottom[1], left_top[1])
                pokemons_allowed_in_field = [pokemon for pokemon in Pokemons.values() if pokemon_field.name  in pokemon.areas]
                pokemon = random.choice(pokemons_allowed_in_field)

                pokemon_sprite = WildPokemon(
                image_path=os.path.join(ASSETS_PATH, f"sprites/pokemon/{pokemon.name}"),
                maggots_bounds=bounds,
                scale=2.0,
                name=pokemon.name,
                field = pokemon_field.name
                )
                pokemon_sprite.position = (
                    random.uniform(bounds[0], bounds[1]),
                    random.uniform(bounds[2], bounds[3])
                )
                if not arcade.check_for_collision_with_list(pokemon_sprite, self.walls):
                    self.wild_pokemon_list.append(pokemon_sprite)
                    self.y_sorted_sprites.append(pokemon_sprite)

                self.counter_pokemon =0
            else:
                self.counter_pokemon += 1
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y
        for pokemon in self.wild_pokemon_list:
            if arcade.check_for_collision_with_list(pokemon, self.walls):
                pokemon.center_x -= pokemon.change_x
                pokemon.center_y -= pokemon.change_y

        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)
        collided_pokemon_list = arcade.check_for_collision_with_list(self.player, self.wild_pokemon_list)
        if collided_pokemon_list:
            collided_pokemon: WildPokemon = collided_pokemon_list[0]
            splash = BattleSplashView(
                # banner_path=os.path.join(ASSETS_PATH, "sprites/pokemon/banner.jpg"),
                overworld_view=self,
                wild_pokemon=collided_pokemon
            )
            self.window.show_view(splash)
            self.wild_pokemon_list.remove(collided_pokemon)
            self.y_sorted_sprites.remove(collided_pokemon)
            self.player.change_x = 0
            self.player.change_y = 0

        # Before sorting:
        for sprite in self.y_sorted_sprites:
            if hasattr(sprite, "_hit_box") and sprite._hit_box:
                # Use bottom of hitbox relative to center_y
                min_hitbox_y = min(point[1] for point in sprite._hit_box.points)
                sprite.depth_y = sprite.center_y + min_hitbox_y
            else:
                sprite.depth_y = sprite.center_y

        self.y_sorted_sprites.sort(key=lambda s: -getattr(s, "depth_y", s.center_y))
        # TODO player grass is to small due to hitbox (probly a new special varibel needed instead of _hit_box
        # TODO refactor on_update. in seperate functions
        # TODO refactoer all forloops gone. god speed.

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
            # TODO move to player? so this can be in update nothing more.

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
if __name__ == "__main__":
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    view = OverworldView()
    view.setup()
    window.show_view(view)
    arcade.run()
