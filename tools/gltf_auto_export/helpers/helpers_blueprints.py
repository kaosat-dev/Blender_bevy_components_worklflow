
import os
from types import SimpleNamespace
import bpy

class Blueprint:
    def __init__(self, name):
        self.name = name
        self.local = True
        self.scene = None # Not sure, could be usefull for tracking

        self.instances = []
        self.objects = []
        self.nested_blueprints = []

        self.collection = None # should we just sublclass ?
    
    def __repr__(self):
        return f'Name: {self.name} Local: {self.local}, Scene: {self.scene}, Instances: {self.instances},  Objects: {self.objects}, nested_blueprints: {self.nested_blueprints}'

    def __str__(self):
        return f'Name: "{self.name}", Local: {self.local}, Scene: {self.scene}, Instances: {self.instances},  Objects: {self.objects}, nested_blueprints: {self.nested_blueprints}'


def find_blueprints_not_on_disk(blueprints, folder_path, extension):
    not_found_blueprints = []
    for blueprint in blueprints:
        gltf_output_path = os.path.join(folder_path, blueprint.name + extension)
        # print("gltf_output_path", gltf_output_path)
        found = os.path.exists(gltf_output_path) and os.path.isfile(gltf_output_path)
        if not found:
            not_found_blueprints.append(blueprint)
    return not_found_blueprints

def check_if_blueprint_on_disk(scene_name, folder_path, extension):
    gltf_output_path = os.path.join(folder_path, scene_name + extension)
    found = os.path.exists(gltf_output_path) and os.path.isfile(gltf_output_path)
    print("level", scene_name, "found", found, "path", gltf_output_path)
    return found

# blueprints: any collection with either
# - an instance
# - marked as asset
# - with the "auto_export" flag
# https://blender.stackexchange.com/questions/167878/how-to-get-all-collections-of-the-current-scene
def blueprints_scan(main_scenes, library_scenes, addon_prefs):
    export_marked_assets = getattr(addon_prefs,"export_marked_assets")

    blueprints = {}
    blueprints_from_objects = {}
    collections = []
    
    # main scenes
    blueprint_instances_per_main_scene = {}
    internal_collection_instances = {}
    external_collection_instances = {}

    for scene in main_scenes:# should it only be main scenes ? what about collection instances inside other scenes ?
        print("scene", scene, scene.name)
        for object in scene.objects:
            #print("object", object.name)
            if object.instance_type == 'COLLECTION':
                collection = object.instance_collection
                collection_name = object.instance_collection.name
                #print("  from collection:", collection_name)
                print("scene there", scene, scene.name)

                collection_from_library = False
                for library_scene in library_scenes: # should be only in library scenes
                    collection_from_library = library_scene.user_of_id(collection) > 0 # TODO: also check if it is an imported asset
                    if collection_from_library:
                        break

                collection_category = internal_collection_instances if collection_from_library else external_collection_instances 
                if not collection_name in collection_category.keys():
                    #print("ADDING INSTANCE OF", collection_name, "object", object.name, "categ", collection_category)
                    collection_category[collection_name] = [] #.append(collection_name)
                collection_category[collection_name].append(object)

                # experiment with custom properties from assets stored in other blend files
                """if not collection_from_library:
                    for property_name in object.keys():
                        print("stuff", property_name)
                    for property_name in collection.keys():
                        print("OTHER", property_name)"""

                # blueprints[collection_name].instances.append(object)

                print("bla", scene.name)
                # FIXME: this only account for direct instances of blueprints, not for any nested blueprint inside a blueprint
                if scene.name not in blueprint_instances_per_main_scene.keys():
                    blueprint_instances_per_main_scene[scene.name] = []
                blueprint_instances_per_main_scene[scene.name].append(collection_name)
                
                """# add any indirect ones
                # FIXME: needs to be recursive, either here or above
                for nested_blueprint in blueprints[collection_name].nested_blueprints:
                    if not nested_blueprint in blueprint_instances_per_main_scene[scene.name]:
                        blueprint_instances_per_main_scene[scene.name].append(nested_blueprint)"""

    for collection in bpy.data.collections: 
        #print("collection", collection, collection.name_full, "users", collection.users)

        collection_from_library = False
        defined_in_scene = None
        for scene in library_scenes: # should be only in library scenes
            collection_from_library = scene.user_of_id(collection) > 0
            if collection_from_library:
                defined_in_scene = scene
                break
        if not collection_from_library: 
            continue

        
        if (
            'AutoExport' in collection and collection['AutoExport'] == True # get marked collections
            or export_marked_assets and collection.asset_data is not None # or if you have marked collections as assets you can auto export them too
            or collection.name in list(internal_collection_instances.keys()) # or if the collection has an instance in one of the main scenes
            ):
            blueprint = Blueprint(collection.name)
            blueprint.local = True
            blueprint.objects = [object.name for object in collection.all_objects if not object.instance_type == 'COLLECTION'] # inneficient, double loop
            blueprint.nested_blueprints = [object.instance_collection.name for object in collection.all_objects if object.instance_type == 'COLLECTION'] # FIXME: not precise enough, aka "what is a blueprint"
            blueprint.collection = collection
            blueprint.instances = internal_collection_instances[collection.name] if collection.name in internal_collection_instances else []
            blueprint.scene = defined_in_scene
            blueprints[collection.name] = blueprint

            # now create reverse lookup , so you can find the collection from any of its contained objects
            for object in collection.all_objects:
                blueprints_from_objects[object.name] = blueprint#collection.name

        #
        collections.append(collection)

    # add any collection that has an instance in the main scenes, but is not present in any of the scenes (IE NON LOCAL)
    for collection_name in external_collection_instances:
        collection = bpy.data.collections[collection_name]
        blueprint = Blueprint(collection.name)
        blueprint.local = False
        blueprint.objects = [object.name for object in collection.all_objects if not object.instance_type == 'COLLECTION'] # inneficient, double loop
        blueprint.nested_blueprints = [object.instance_collection.name for object in collection.all_objects if object.instance_type == 'COLLECTION'] # FIXME: not precise enough, aka "what is a blueprint"
        blueprint.collection = collection
        blueprint.instances = external_collection_instances[collection.name] if collection.name in external_collection_instances else []

        blueprints[collection.name] = blueprint

        # now create reverse lookup , so you can find the collection from any of its contained objects
        for object in collection.all_objects:
            blueprints_from_objects[object.name] = blueprint#collection.name


    # then add any nested collections at root level (so we can have a flat list, regardless of nesting)
    # TODO: do this recursively
    for blueprint_name in list(blueprints.keys()):
        parent_blueprint = blueprints[blueprint_name]
        for nested_blueprint_name in parent_blueprint.nested_blueprints:
            if not nested_blueprint_name in blueprints.keys():
                collection = bpy.data.collections[nested_blueprint_name]
                blueprint = Blueprint(collection.name)
                blueprint.local = parent_blueprint.local
                blueprint.objects = [object.name for object in collection.all_objects if not object.instance_type == 'COLLECTION'] # inneficient, double loop
                blueprint.nested_blueprints = [object.instance_collection.name for object in collection.all_objects if object.instance_type == 'COLLECTION'] # FIXME: not precise enough, aka "what is a blueprint"
                blueprint.collection = collection
                blueprint.instances = external_collection_instances[collection.name] if collection.name in external_collection_instances else []
                blueprint.scene = parent_blueprint.scene if parent_blueprint.local else None
                blueprints[collection.name] = blueprint

                # now create reverse lookup , so you can find the collection from any of its contained objects
                for object in collection.all_objects:
                    blueprints_from_objects[object.name] = blueprint#collection.name


    blueprints = dict(sorted(blueprints.items()))

    print("BLUEPRINTS")
    for blueprint_name in blueprints:
        print(" ", blueprints[blueprint_name])

    """print("BLUEPRINTS LOOKUP")
    print(blueprints_from_objects)"""

    print("BLUEPRINT INSTANCES PER MAIN SCENE")
    print(blueprint_instances_per_main_scene)


    """changes_test = {'Library': {
        'Blueprint1_mesh': bpy.data.objects['Blueprint1_mesh'], 
        'Fox_mesh': bpy.data.objects['Fox_mesh'],
        'External_blueprint2_Cylinder': bpy.data.objects['External_blueprint2_Cylinder']}
    }
    # which main scene has been impacted by this
    # does one of the main scenes contain an INSTANCE of an impacted blueprint
    for scene in main_scenes:
        changed_objects = list(changes_test["Library"].keys()) # just a hack for testing
        #bluprint_instances_in_scene = blueprint_instances_per_main_scene[scene.name]
        #print("instances per scene", bluprint_instances_in_scene, "changed_objects", changed_objects)

        changed_blueprints_with_instances_in_scene = [blueprints_from_objects[changed] for changed in changed_objects if changed in blueprints_from_objects]
        print("changed_blueprints_with_instances_in_scene", changed_blueprints_with_instances_in_scene)
        level_needs_export = len(changed_blueprints_with_instances_in_scene) > 0
        if level_needs_export:
            print("level needs export", scene.name)

    for scene in library_scenes:
        changed_objects = list(changes_test[scene.name].keys())
        changed_blueprints = [blueprints_from_objects[changed] for changed in changed_objects if changed in blueprints_from_objects]
        # we only care about local blueprints/collections
        changed_local_blueprints = [blueprint_name for blueprint_name in changed_blueprints if blueprint_name in blueprints.keys() and blueprints[blueprint_name].local]
        print("changed blueprints", changed_local_blueprints)"""

    # additional helper data structures for lookups etc
    blueprints_per_name = blueprints
    blueprints = [] # flat list
    internal_blueprints = []
    external_blueprints = []
    blueprints_per_scenes = {}
    for blueprint in blueprints_per_name.values():
        blueprints.append(blueprint)
        if blueprint.local:
            internal_blueprints.append(blueprint)
            if blueprint.scene:
                if not blueprint.scene.name in blueprints_per_scenes:
                    blueprints_per_scenes[blueprint.scene.name] = []
                blueprints_per_scenes[blueprint.scene.name].append(blueprint.name) # meh
        else:
            external_blueprints.append(blueprint)

    data = {
        "blueprints": blueprints,
        "blueprints_per_name": blueprints_per_name,
        "blueprint_names": list(blueprints_per_name.keys()),
        "blueprints_from_objects": blueprints_from_objects,

        "internal_blueprints": internal_blueprints,
        "external_blueprints": external_blueprints,
        "blueprints_per_scenes": blueprints_per_scenes,

        "blueprint_instances_per_main_scene": blueprint_instances_per_main_scene
        # not sure about these two
        #internal_collection_instances,
        #external_collection_instances
    }

    return SimpleNamespace(**data)


import json
from .object_makers import (make_empty)

def inject_blueprints_list_into_main_scene(scene, blueprints_data):
    print("injecting assets/blueprints data into scene")
    root_collection = scene.collection
    assets_list = None
    assets_list_name = f"assets_list_{scene.name}_components"
    for object in scene.objects:
        if object.name == assets_list_name:
            assets_list = object
            break

    if assets_list is None:
        assets_list = make_empty(assets_list_name, [0,0,0], [0,0,0], [0,0,0], root_collection)

    blueprint_names_for_scene = blueprints_data.blueprint_instances_per_main_scene.get(scene.name, None)
    # find all blueprints used in a scene
    if blueprint_names_for_scene: # what are the blueprints used in this scene, inject those into the assets list component
        print("blueprint_names_for_scene", blueprint_names_for_scene)
        children_per_blueprint = {}
        for blueprint_name in blueprint_names_for_scene:
            blueprint = blueprints_data.blueprints_per_name.get(blueprint_name, None)
            if blueprint:
                children_per_blueprint[blueprint_name] = blueprint.nested_blueprints
        print("new logic blueprints list", children_per_blueprint)
        assets_list["BlueprintsList"] = f"({json.dumps(dict(children_per_blueprint))})"

def remove_blueprints_list_from_main_scene(scene):
    assets_list = None
    assets_list_name = f"assets_list_{scene.name}_components"

    for object in scene.objects:
        if object.name == assets_list_name:
            assets_list = object
    if assets_list is not None:
        bpy.data.objects.remove(assets_list, do_unlink=True)
