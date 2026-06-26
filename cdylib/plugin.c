#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

#include "flecs.h"
#include <stdio.h>

typedef struct RandomNumber {
    short value;
} RandomNumber;

typedef struct PluginApi {
    // World
    ecs_world_t* world;
    // Labels
    ecs_entity_t Update;
    ecs_entity_t Render;
    // Components
    ecs_entity_t RandomNumber;
} PluginApi;

static void update_system(ecs_iter_t* it) {
    printf("update from c\n");
}

static void print_random(ecs_iter_t* it) {
    RandomNumber *num = ecs_field(it, RandomNumber, 0);

    for (int i = 0; i < it->count; i++) {
        printf("Random number: %d\n", num[i].value);
    }
}

static void render_system(ecs_iter_t* it) {
    printf("render from c\n");
}

EXPORT void register_systems(const PluginApi* api) {
    ecs_system_init(api->world, &(ecs_system_desc_t){
        .entity = ecs_entity(api->world, {
            .name = "CUpdate"
        }),
        .callback = update_system,
        .query = {
            .terms = {
                { .id = api->RandomNumber }
            }
        },
        .phase = api->Update
    });

    ecs_system_init(api->world, &(ecs_system_desc_t){
        .entity = ecs_entity(api->world, {
            .name = "CRender"
        }),
        .callback = render_system,
        .phase = api->Render
    });

    ecs_system_init(api->world, &(ecs_system_desc_t){
        .entity = ecs_entity(api->world, {
            .name = "PrintRandom"
        }),
        .callback = print_random,
        .query = {
            .terms = {
                { .id = api->RandomNumber }
            }
        },
        .phase = api->Render
    });
}