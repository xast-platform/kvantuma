CC=gcc

# must match flecs_ecs's default Cargo feature set, or struct layouts diverge from Rust's
FLECS_DEFINES=-DFLECS_CUSTOM_BUILD -DFLECS_CPP -DFLECS_TERM_COUNT_MAX=32 \
	-DFLECS_MODULE -DFLECS_SYSTEM -DFLECS_PIPELINE -DFLECS_TIMER \
	-DFLECS_OS_API_IMPL -DFLECS_MUT_ALIAS_LOCKS -DFLECS_SCRIPT \
	-DFLECS_STATS -DFLECS_METRICS -DFLECS_ALERTS -DFLECS_META \
	-DFLECS_META_C -DFLECS_UNITS -DFLECS_JSON -DFLECS_DOC -DFLECS_LOG \
	-DFLECS_APP -DFLECS_HTTP -DFLECS_REST

all: libplugin.so

# no flecs.c here: ecs_* resolves at dlopen against the 
# engine's own flecs (-rdynamic), avoiding a second flecs instance
libplugin.so: plugin.c flecs.h
	$(CC) -g -O0 $(FLECS_DEFINES) -shared -fPIC plugin.c -o ../assets/plugins/libtestplugin.so
	
clean:
	rm -f libplugin.so host