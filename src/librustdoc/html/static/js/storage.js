var darkThemes = ["dark", "ayu"];

var settingsDataset = (function () {
    var settingsElement = document.getElementById("default-settings");
    if (settingsElement === null) {
        return null;
    }
    var dataset = settingsElement.dataset;
    if (dataset === undefined) {
        return null;
    }
    return dataset;
})();

function getSettingValue(settingName) {
    var current = getCurrentValue('rustdoc-' + settingName);
    if (current !== null) {
        return current;
    }
    if (settingsDataset !== null) {
        // See the comment for `default_settings.into_iter()` etc. in
        // `Options::from_matches` in `librustdoc/config.rs`.
        var def = settingsDataset[settingName.replace(/-/g,'_')];
        if (def !== undefined) {
            return def;
        }
    }
    return null;
}

// eslint-disable-next-line no-unused-vars
function hasClass(elem, className) {
    return elem && elem.classList && elem.classList.contains(className);
}

// eslint-disable-next-line no-unused-vars
function addClass(elem, className) {
    if (!elem || !elem.classList) {
        return;
    }
    elem.classList.add(className);
}

// eslint-disable-next-line no-unused-vars
function removeClass(elem, className) {
    if (!elem || !elem.classList) {
        return;
    }
    elem.classList.remove(className);
}

function onEach(arr, func, reversed) {
    if (arr && arr.length > 0 && func) {
        var length = arr.length;
        var i;
        if (reversed) {
            for (i = length - 1; i >= 0; --i) {
                if (func(arr[i])) {
                    return true;
                }
            }
        } else {
            for (i = 0; i < length; ++i) {
                if (func(arr[i])) {
                    return true;
                }
            }
        }
    }
    return false;
}

function onEachLazy(lazyArray, func, reversed) {
    return onEach(
        Array.prototype.slice.call(lazyArray),
        func,
        reversed);
}

// eslint-disable-next-line no-unused-vars
function hasOwnPropertyRustdoc(obj, property) {
    return Object.prototype.hasOwnProperty.call(obj, property);
}

function updateLocalStorage(name, value) {
    try {
        window.localStorage.setItem(name, value);
    } catch(e) {
        // localStorage is not accessible, do nothing
    }
}

function getCurrentValue(name) {
    try {
        return window.localStorage.getItem(name);
    } catch(e) {
        return null;
    }
}

function themeURL(themeName) {
    var mainThemeURL = document.getElementById("mainThemeStyle").href;
    return mainThemeURL.replace(/rustdoc([^/]*)\.css/, themeName + "$1.css");
}

function useOneTheme(value) {
    updateLocalStorage("rustdoc-theme", value);
    useSystemTheme(false);
    applyThemeUpdate();
}

// This function is called from "main.js".
// eslint-disable-next-line no-unused-vars
function useSystemTheme(value) {
    if (value === undefined) {
        value = true;
    }

    updateLocalStorage("rustdoc-use-system-theme", value);

    // update the toggle if we're on the settings page
    var toggle = document.getElementById("use-system-theme");
    if (toggle && toggle instanceof HTMLInputElement) {
        toggle.checked = value;
    }
}

// Returns "light" or "dark" depending on the user's browser preferences
// (that is, independent of any rustdoc settings).
function getSystemTheme() {
    if (!window.matchMedia) {
      // In rustdoc.css we have:
      // @media (prefers-color-scheme: light) {
      //     html {
      //         content: "light";
      //     }
      // }
      // @media (prefers-color-scheme: dark) {
      //     html {
      //         content: "dark";
      //     }
      // }

      return getComputedStyle(document.documentElement)
          .getPropertyValue('content') || "light";
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        return "dark";
    } else {
        return "light";
    }
}

// If there is a specific theme selected, or if "Use System Theme" is selected
// and there is a specific theme for the current light or dark mode, return
// the appropriate theme's name. Otherwise, return null.
function getThemeOverride() {
    if (getSettingValue("use-system-theme") !== "false") {
        var lightOverride = getSettingValue("preferred-light-theme");
        var darkOverride = getSettingValue("preferred-dark-theme");

        var preferred = getSystemTheme();
        if (preferred == "light" && lightOverride && lightOverride != "light") {
            console.log("lightoverride ", lightOverride);
            return lightOverride;
        } else if (preferred == "dark" && darkOverride && darkOverride != "dark") {
            console.log("darkoverride ", darkOverride)
            return darkOverride;
        } else {
            console.log("use default");
            return null;
        }
    } else {
        console.log("getsettingvalue ", getSettingValue("theme"));
        return getSettingValue("theme") || null;
    }
}

const nonDefaultThemeID = "nonDefaultThemeStyle";

function applyThemeUpdate() {
    var lightOverride = getSettingValue("preferred-light-theme");
    var darkOverride = getSettingValue("preferred-dark-theme");
    var allOverride = getSettingValue("theme");
    if (allOverride) {
        document.getElementById(nonDefaultThemeID).href = themeURL(allOverride);
    } else {
        if (lightOverride) {
            document.getElementById("lightThemeStyle").href = themeURL(lightOverride);
        }
        if (darkOverride) {
            document.getElementById("darkThemeStyle").href = themeURL(darkOverride);
        }
        document.getElementById(nonDefaultThemeID).href = "";
    }
}

function updatePreferredDarkTheme() {
    // update the preferred dark theme if the user is already using a dark theme
    // See https://github.com/rust-lang/rust/pull/77809#issuecomment-707875732
    var localStoredTheme = getSettingValue("theme");
    if (getSettingValue("use-system-theme") === null
        && getSettingValue("preferred-dark-theme") === null
        && darkThemes.indexOf(localStoredTheme) >= 0) {
        updateLocalStorage("rustdoc-preferred-dark-theme", localStoredTheme);
    }
}

// If there's a theme override that's not covered by one of the default
// dark / light themes, we write it to the document head. We use
// document.write rather than appendChild intentionally. Stylesheets added
// with document.write are render-blocking, while those added with appendChild
// are not. We want render blocking behavior to avoid a flash of unstyled
// content.
function applyThemeOverride() {
    var override = getThemeOverride();
    if (override) {
        document.write('<link rel="stylesheet" type="text/css" id="' + nonDefaultThemeID +
           '" href="' + themeURL(override) + '">');
    } else {
        document.write('<link rel="stylesheet" type="text/css" id="' + nonDefaultThemeID + '">');
    }
}

updatePreferredDarkTheme();
applyThemeOverride();
if (window.matchMedia) {
    var mql = window.matchMedia("(prefers-color-scheme: dark)");
    mql.addEventListener('change', applyThemeUpdate);
}
