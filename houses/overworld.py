import random
from pathlib import Path

import arcade

from houses.BaseMap import BaseMap
from initate_battle import BattleSplashView
from lifeforms.pokemons import WildPokemon
from pokemon import Pokemons
from utils import ASSETS_PATH


class OverworldView(BaseMap):
    def __init__(self,player_location_key:str,map:Path,possible_gates:list[str]):
        self.max_pokemon_per_bush = 3 # TODO: calculate this with surface area of the field.
        self.counter_pokemon =0
        self.pokemon_fields = {}
        self.wild_pokemon_list = arcade.SpriteList()
        super().__init__(player_location_key=player_location_key,map=map,possible_gates=possible_gates)


    def setup(self):
        super().setup()
        self.y_sorted_sprites.extend(self.scene["grass"])
        self.y_sorted_sprites.extend(self.wild_pokemon_list)

    def on_update(self, delta_time):
        super().on_update(delta_time)
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
                image_path=ASSETS_PATH/ f"sprites/pokemon/{pokemon.name}",
                maggots_bounds=bounds,
                scale=2.0,
                name=pokemon.name,
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
        for pokemon in self.wild_pokemon_list:
            if arcade.check_for_collision_with_list(pokemon, self.walls):
                pokemon.center_x -= pokemon.change_x
                pokemon.center_y -= pokemon.change_y
            else:
                pokemon.update_animation()

        collided_pokemon_list = arcade.check_for_collision_with_list(self.player, self.wild_pokemon_list)
        if collided_pokemon_list:
            collided_pokemon: WildPokemon = collided_pokemon_list[0]
            splash = BattleSplashView(
                overworld_view=self,
                wild_pokemon=collided_pokemon
            )
            self.window.show_view(splash)
            self.wild_pokemon_list.remove(collided_pokemon)
            self.y_sorted_sprites.remove(collided_pokemon)
            self.player.change_x = 0
            self.player.change_y = 0




        # TODO player grass is to small due to hitbox (probly a new special varibel needed instead of _hit_box
        # TODO refactor on_update. in seperate functions
        # TODO refactoer all forloops gone. god speed.
