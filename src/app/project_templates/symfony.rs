use std::fs;
use std::path::Path;

pub(crate) fn generate(
    _name: &str,
    path: &Path,
    version: &str,
    php_version: &str,
) -> Result<(), String> {
    let composer_json = format!(
        r#"{{
    "type": "project",
    "license": "MIT",
    "minimum-stability": "stable",
    "prefer-stable": true,
    "require": {{
        "php": ">={1}",
        "ext-ctype": "*",
        "ext-iconv": "*",
        "symfony/console": "{0}",
        "symfony/dotenv": "{0}",
        "symfony/flex": "^2",
        "symfony/framework-bundle": "{0}",
        "symfony/runtime": "{0}",
        "symfony/yaml": "{0}"
    }},
    "config": {{
        "allow-plugins": {{
            "php-http/discovery": true,
            "symfony/flex": true,
            "symfony/runtime": true
        }},
        "sort-packages": true
    }},
    "autoload": {{
        "psr-4": {{
            "App\\": "src/"
        }}
    }},
    "autoload-dev": {{
        "psr-4": {{
            "App\\Tests\\": "tests/"
        }}
    }},
    "replace": {{
        "symfony/polyfill-ctype": "*",
        "symfony/polyfill-iconv": "*",
        "symfony/polyfill-php72": "*",
        "symfony/polyfill-php73": "*",
        "symfony/polyfill-php74": "*",
        "symfony/polyfill-php80": "*",
        "symfony/polyfill-php81": "*",
        "symfony/polyfill-php82": "*"
    }},
    "scripts": {{
        "auto-scripts": {{
            "cache:clear": "symfony-cmd",
            "assets:install %PUBLIC_DIR%": "symfony-cmd"
        }},
        "post-install-cmd": [
            "@auto-scripts"
        ],
        "post-update-cmd": [
            "@auto-scripts"
        ]
    }},
    "conflict": {{
        "symfony/symfony": "*"
    }},
    "extra": {{
        "symfony": {{
            "allow-contrib": false,
            "require": "{0}"
        }}
    }}
}}"#,
        version, php_version
    );

    let index_php = r#"<?php
use App\Kernel;
require_once dirname(__DIR__).'/vendor/autoload_runtime.php';
return function (array $context) {
    return new Kernel($context['APP_ENV'], (bool) $context['APP_DEBUG']);
};
"#;

    let console_bin = r#"#!/usr/bin/env php
<?php
use App\Kernel;
use Symfony\Bundle\FrameworkBundle\Console\Application;
use Symfony\Component\Console\Input\ArgvInput;
use Symfony\Component\ErrorHandler\Debug;

if (!file_exists(dirname(__DIR__).'/vendor/autoload_runtime.php')) {
    throw new LogicException('Symfony Runtime is missing. Try running "composer install".');
}

require_once dirname(__DIR__).'/vendor/autoload_runtime.php';

return function (array $context) {
    if ($context['APP_DEBUG']) {
        umask(0000);
        Debug::enable();
    }
    $kernel = new Kernel($context['APP_ENV'], (bool) $context['APP_DEBUG']);
    return new Application($kernel);
};
"#;

    let kernel_php = r#"<?php
namespace App;
use Symfony\Bundle\FrameworkBundle\Kernel\MicroKernelTrait;
use Symfony\Component\HttpKernel\Kernel as BaseKernel;

class Kernel extends BaseKernel
{
    use MicroKernelTrait;
}
"#;

    fs::create_dir_all(path.join("bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("config")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("public")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("var")).map_err(|e| e.to_string())?;

    fs::write(path.join("composer.json"), composer_json).map_err(|e| e.to_string())?;
    fs::write(path.join("public/index.php"), index_php).map_err(|e| e.to_string())?;
    fs::write(path.join("bin/console"), console_bin).map_err(|e| e.to_string())?;
    fs::write(path.join("src/Kernel.php"), kernel_php).map_err(|e| e.to_string())?;
    fs::write(path.join(".env"), "APP_ENV=dev\nAPP_SECRET=change_me\n")
        .map_err(|e| e.to_string())?;
    fs::write(path.join(".gitignore"), "/vendor/\n/var/\n.env.local\n")
        .map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path.join("bin/console")) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(path.join("bin/console"), perms);
        }
    }

    Ok(())
}
