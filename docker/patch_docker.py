import json
from pathlib import Path

from pygments.lexers import shell
from typer import Typer
import subprocess

app = Typer()

directory = Path(__file__).parent / 'docker-postgres'


@app.command()
def patch_docker():
    versions_file = directory / 'versions.json'
    versions = json.loads(versions_file.read_text())

    target_variant = 'bookworm'

    keys_to_keep = {target_variant, 'major', 'sha256', 'version'}

    for version, info in versions.items():
        new_info = {
            key: info[key]
            for key in keys_to_keep
        }
        # Force compilation
        new_info[target_variant]['arches'] = ['does-not-exist']
        new_info['variants'] = [target_variant]
        versions[version] = new_info
    import pprint
    pprint.pprint(versions)
    versions_file.write_text(json.dumps(versions, indent=2, sort_keys=True))
    subprocess.run(
        './apply-templates.sh',
        shell=True,
        cwd=directory.absolute()
    )
    dockerfiles = list(directory.glob('*/*/Dockerfile'))
    if not dockerfiles:
        raise RuntimeError('No Dockerfiles found after processing!')

    package_cflags = "DEB_CFLAGS_MAINT_APPEND='-DLOCK_DEBUG=1'"
    append_after = f'FROM debian:{target_variant}-slim'

    #line_to_patch = '\t\t\tapt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"'
    #replacement = '\t\t\tapt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"'

    for dockerfile in dockerfiles:
        dockerfile_contents: str = dockerfile.read_text()
        if package_cflags in dockerfile_contents:
            print(f'{dockerfile} already patched')
            continue

        if append_after not in dockerfile_contents:
            raise RuntimeError(f'{dockerfile} does not contain "{append_after}"!')

        dockerfile_contents = dockerfile_contents.replace(
            append_after,
            f'{append_after}\nENV {package_cflags}'
        )
        dockerfile.write_text(dockerfile_contents)

        # subprocess.run(
        #     f'docker build -t postgres:{dockerfile.suffix[1:]} -f {dockerfile} .',
        #     shell=True,
        #     cwd=directory
        # )


if __name__ == '__main__':
    app()
