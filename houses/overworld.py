import random
from pathlib import Path

import arcade
from arcade.types import TiledObject
from loguru import logger

from houses.BaseMap import BaseMap, DialogMonsterPanel
from houses.helper_stuff import check_object_collision
from initate_battle import BattleSplashView
from lifeforms.pokemons import WildPokemon
from pokemon import Monsters
from utils import load_state, save_state


class OverworldView(BaseMap):

    def __init__(self, map: Path, possible_gates: list[str], music: Path):
        self.max_pokemon_per_bush = (
            3  # TODO: calculate this with surface area of the field.
        )
        self.counter_pokemon = 0
        self.wild_pokemon_list = arcade.SpriteList()
        super().__init__(
            map=map, possible_gates=possible_gates, music=music, current_player=0.2
        )

    def setup(self):
        super().setup()
        self.y_sorted_sprites.extend(self.scene["grass"])
        self.y_sorted_sprites.extend(self.wild_pokemon_list)

    def on_update(self, delta_time):
        super().on_update(delta_time)
        if self.dialog:
            pokemon_defeated = [
                pokemon for pokemon in self.wild_pokemon_list if not pokemon.alive
            ][0]
            pokemon_defeated.update(delta_time)
            if not self.dialog.active:
                if self.dialog.download_monster:
                    logger.debug("downloading...")
                    self.wild_pokemon_list.remove(pokemon_defeated)
                    self.y_sorted_sprites.remove(pokemon_defeated)
                    self.player_state["pokemons"][pokemon_defeated.name.lower()] = {
                        "attacks": {
                            "base": "tacle",
                            "down": None,
                            "left": None,
                            "right": None,
                            "up": None,
                        },
                        "stats": {
                            "attack": 50,
                            "defense": 20,
                            "health": 100,
                            "speed": 10,
                        },
                    }
                    # TODO select random setup of Monsters
                    save_state(self.player_state)

                if self.dialog.kill_monster:
                    self.wild_pokemon_list.remove(pokemon_defeated)
                    self.y_sorted_sprites.remove(pokemon_defeated)
                self.dialog = None
            return
        self.wild_pokemon_list.update()
        pokemon_fields = self.tile_map.object_lists["pokemonFields"]
        player_in_pokemon_field = next(
            (
                pokemon_field
                for pokemon_field in pokemon_fields
                if check_object_collision(self.player, pokemon_field, padding=80)
            ),
            None,
        )
        if (
            self.max_pokemon_per_bush > len(self.wild_pokemon_list)
            and player_in_pokemon_field is not None
        ):
            if self.counter_pokemon > random.uniform(100, 500):
                self.spawn_pokemon(player_in_pokemon_field)
            else:
                self.counter_pokemon += 1
        elif player_in_pokemon_field is None:
            self.wild_pokemon_list.clear()
            self.y_sorted_sprites.clear()
        for pokemon in self.wild_pokemon_list:
            if arcade.check_for_collision_with_list(
                pokemon, self.walls
            ) or not arcade.check_for_collision_with_list(pokemon, self.scene["grass"]):
                pokemon.center_x -= pokemon.change_x
                pokemon.center_y -= pokemon.change_y
            else:
                pokemon.update_animation()

        collided_pokemon_list = arcade.check_for_collision_with_list(
            self.player, self.wild_pokemon_list
        )
        if collided_pokemon_list:
            collided_pokemon: WildPokemon = collided_pokemon_list[0]
            if collided_pokemon.alive:
                splash = BattleSplashView(
                    overworld_view=self, wild_pokemon=collided_pokemon
                )
                self.window.show_view(splash)
                self.player.change_x = 0
                self.player.change_y = 0

    def spawn_pokemon(self, pokemon_field: TiledObject):
        left_top, right_top, right_bottom, left_bottom = pokemon_field.shape
        bounds = (left_top[0], right_top[0], left_bottom[1], left_top[1])
        pokemons_allowed_in_field = [
            pokemon for pokemon in Monsters if int(pokemon_field.name) in pokemon.fields
        ]
        pokemon = random.choice(pokemons_allowed_in_field)

        pokemon_sprite = pokemon.over_world
        pokemon_sprite.position = (
            random.uniform(bounds[0], bounds[1]),
            random.uniform(bounds[2], bounds[3]),
        )
        if not arcade.check_for_collision_with_list(pokemon_sprite, self.walls):
            if not pokemon.name in self.player_state["pokedex"]:
                logger.debug("found a new pokemon!")
                self.player_state["pokedex"].append(pokemon.name)
                save_state(self.player_state)
            self.wild_pokemon_list.append(pokemon_sprite)
            self.y_sorted_sprites.append(pokemon_sprite)

        self.counter_pokemon = 0

    def player_lost(self):
        logger.debug("dood!")
        self.wild_pokemon_list.clear()
        self.y_sorted_sprites.clear()
        load_state()
        self.window.show_view(self)

    def enemy_out_of_bounds(self, wild_pokemon: WildPokemon):
        self.wild_pokemon_list.remove(wild_pokemon)
        self.y_sorted_sprites.remove(wild_pokemon)
        self.window.show_view(self)

    def enemy_defeated(self, wild_pokemon: WildPokemon):
        wild_pokemon.alive = False
        self.window.show_view(self)
        self.dialog = DialogMonsterPanel(self.player_state)
