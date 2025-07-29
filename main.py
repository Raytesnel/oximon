import os
import random

import arcade
from arcade import load_tilemap

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
        self.max_pokemon_per_bush = 3 # TODO: take this from the tiled map.
        self.counter_pokemon =0
        self.pokemon_fields = {}
        self.all_sprites = []

    def setup(self):
        self.player_list = arcade.SpriteList()
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.camera = arcade.Camera2D()
        self.tile_map = load_tilemap(
            os.path.join(ASSETS_PATH, "map/probeersel_01.tmx"),
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
        self.y_sorted_sprites.append(self.player)

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.wild_pokemon_list.draw()
        self.y_sorted_sprites.draw()
        self.scene.get_sprite_list("voorgrond").draw()
        # self.player.draw_hit_box(arcade.color.RED)

    def on_update(self, delta_time):
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        self.wild_pokemon_list.update()
        pokemon_fields = self.tile_map.object_lists["pokemonFields"]
        pokemon_field = random.choice(pokemon_fields)
        if pokemon_field.name not in self.pokemon_fields.keys():
            self.pokemon_fields[pokemon_field.name] = []
        pokemons_in_field =[pokemon for pokemon in self.wild_pokemon_list if pokemon_field.name == pokemon.field]
        if self.max_pokemon_per_bush > len(pokemons_in_field):
            if self.counter_pokemon > random.uniform(100,5000):
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
                self.wild_pokemon_list.append(pokemon_sprite)
                self.counter_pokemon =0
            else:
                self.counter_pokemon += 1
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y

        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)
        collided_pokemon_list = arcade.check_for_collision_with_list(self.player, self.wild_pokemon_list)
        if collided_pokemon_list:
            collided_pokemon: WildPokemon = collided_pokemon_list[0]
            splash = BattleSplashView(
                player_sprite_path=os.path.join(ASSETS_PATH, "sprites/pokemon/player_shot.png"),
                banner_path=os.path.join(ASSETS_PATH, "sprites/pokemon/banner.jpg"),
                game_view=self,
                wild_pokemon=collided_pokemon
            )
            self.window.show_view(splash)
            self.wild_pokemon_list.remove(collided_pokemon)
        self.y_sorted_sprites.sort(key=lambda s: -s.center_y)

        # TODO make reset world view & walking
        # TODO make the grass move between pokemon

    def on_key_press(self, key, modifiers):
        self.player.change_x = 0
        self.player.change_y = 0
        if key == arcade.key.UP:
            self.player.change_y = 5
        elif key == arcade.key.DOWN:
            self.player.change_y = -5
        elif key == arcade.key.LEFT:
            self.player.change_x = -5
        elif key == arcade.key.RIGHT:
            self.player.change_x = 5

    def on_key_release(self, key, modifiers):
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0


if __name__ == "__main__":
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    view = OverworldView()
    view.setup()
    window.show_view(view)
    arcade.run()
