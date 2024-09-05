export const pluginApi = {
    enable() {
        console.log("Enabled!");
        return 1;
    },

    disable() {
        console.log("Disabled");
        return 1;
    },

    processSignal(ptr) {
        console.log("Process Signal!!!");
        return 1;
    }
}