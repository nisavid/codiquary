#define _POSIX_C_SOURCE 200809L

#include <alpm.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define REPOSITORY_NAME "codiquary17"
#define PACKAGE_NAME "codiquary17-native"
#define PACKAGE_VERSION "1.0-1"
#define PACKAGE_FILENAME "codiquary17-native-1.0-1-any.pkg.tar.zst"
#define OUTPUT_SCHEMA "codiquary.pacman-native-probe.v1"

struct question_state {
    size_t calls;
};

struct signature_evidence {
    int check_return;
    size_t count;
    char *fingerprint;
    alpm_sigstatus_t status;
    alpm_sigvalidity_t validity;
};

static void refuse_questions(void *context, alpm_question_t *question)
{
    struct question_state *state = context;

    state->calls++;
    if (question == NULL) {
        return;
    }

    question->any.answer = 0;
    if (question->type == ALPM_QUESTION_IMPORT_KEY) {
        question->import_key.import = 0;
    }
}

static void report_alpm_error(alpm_handle_t *handle, const char *operation)
{
    alpm_errno_t error = alpm_errno(handle);
    fprintf(stderr, "%s: %s (%d)\n", operation, alpm_strerror(error), (int)error);
}

static int capture_signature(const char *label,
        int check_return,
        const alpm_siglist_t *list,
        const char *expected_fingerprint,
        struct signature_evidence *evidence)
{
    const alpm_sigresult_t *result;

    if (check_return != 0) {
        fprintf(stderr, "%s signature check returned %d\n", label, check_return);
        return -1;
    }
    if (list->count != 1) {
        fprintf(stderr, "%s signature count was %zu, expected 1\n", label, list->count);
        return -1;
    }
    if (list->results == NULL) {
        fprintf(stderr, "%s signature result array is null\n", label);
        return -1;
    }

    result = &list->results[0];
    if (result->key.fingerprint == NULL) {
        fprintf(stderr, "%s native result fingerprint is null\n", label);
        return -1;
    }
    if (strcmp(result->key.fingerprint, expected_fingerprint) != 0) {
        fprintf(stderr,
                "%s native result fingerprint was %s, expected %s\n",
                label,
                result->key.fingerprint,
                expected_fingerprint);
        return -1;
    }
    if (result->status != ALPM_SIGSTATUS_VALID) {
        fprintf(stderr, "%s signature status was %d, expected %d\n",
                label, (int)result->status, (int)ALPM_SIGSTATUS_VALID);
        return -1;
    }
    if (result->validity != ALPM_SIGVALIDITY_FULL) {
        fprintf(stderr, "%s signature validity was %d, expected %d\n",
                label, (int)result->validity, (int)ALPM_SIGVALIDITY_FULL);
        return -1;
    }

    evidence->fingerprint = strdup(result->key.fingerprint);
    if (evidence->fingerprint == NULL) {
        fprintf(stderr, "out of memory while copying %s fingerprint\n", label);
        return -1;
    }
    evidence->check_return = check_return;
    evidence->count = list->count;
    evidence->status = result->status;
    evidence->validity = result->validity;
    return 0;
}

static const char *status_name(alpm_sigstatus_t status)
{
    switch (status) {
    case ALPM_SIGSTATUS_VALID:
        return "Valid";
    case ALPM_SIGSTATUS_KEY_EXPIRED:
        return "KeyExpired";
    case ALPM_SIGSTATUS_SIG_EXPIRED:
        return "SignatureExpired";
    case ALPM_SIGSTATUS_KEY_UNKNOWN:
        return "KeyUnknown";
    case ALPM_SIGSTATUS_KEY_DISABLED:
        return "KeyDisabled";
    case ALPM_SIGSTATUS_INVALID:
        return "Invalid";
    }
    return "Unknown";
}

static const char *validity_name(alpm_sigvalidity_t validity)
{
    switch (validity) {
    case ALPM_SIGVALIDITY_FULL:
        return "Full";
    case ALPM_SIGVALIDITY_MARGINAL:
        return "Marginal";
    case ALPM_SIGVALIDITY_NEVER:
        return "Never";
    case ALPM_SIGVALIDITY_UNKNOWN:
        return "Unknown";
    }
    return "Unknown";
}

static void write_json_string(const char *value)
{
    const unsigned char *cursor = (const unsigned char *)value;

    putchar('"');
    while (*cursor != '\0') {
        switch (*cursor) {
        case '"':
            fputs("\\\"", stdout);
            break;
        case '\\':
            fputs("\\\\", stdout);
            break;
        case '\b':
            fputs("\\b", stdout);
            break;
        case '\f':
            fputs("\\f", stdout);
            break;
        case '\n':
            fputs("\\n", stdout);
            break;
        case '\r':
            fputs("\\r", stdout);
            break;
        case '\t':
            fputs("\\t", stdout);
            break;
        default:
            if (*cursor < 0x20) {
                printf("\\u%04x", (unsigned int)*cursor);
            } else {
                putchar((int)*cursor);
            }
            break;
        }
        cursor++;
    }
    putchar('"');
}

static int path_is_absolute(const char *path)
{
    return path != NULL && path[0] == '/';
}

int main(int argc, char **argv)
{
    const char *root;
    const char *dbpath;
    const char *gpgdir;
    const char *package_path;
    const char *package_check_level;
    const char *database_check_level;
    const char *expected_fingerprint;
    alpm_errno_t initialize_error = ALPM_ERR_OK;
    alpm_handle_t *handle = NULL;
    alpm_pkg_t *file_package = NULL;
    alpm_db_t *database = NULL;
    alpm_pkg_t *database_package = NULL;
    alpm_siglist_t package_signature_list = {0};
    alpm_siglist_t database_signature_list = {0};
    struct signature_evidence package_signature = {0};
    struct signature_evidence database_signature = {0};
    struct question_state questions = {0};
    int package_signature_initialized = 0;
    int database_signature_initialized = 0;
    int package_siglevel = -1;
    int requested_database_siglevel = -1;
    int database_siglevel = -1;
    int package_load_return = -1;
    int package_errno_ok = 0;
    int package_non_null = 0;
    int database_non_null = 0;
    int database_valid_return = -1;
    int candidate_ready = 0;
    int cleanup_ok = 1;
    char *database_package_name = NULL;
    char *database_package_version = NULL;
    char *database_package_filename = NULL;

    if (argc != 8) {
        fprintf(stderr,
                "usage: %s ROOT DBPATH GPGDIR PACKAGE PACKAGE_CHECK_LEVEL DATABASE_CHECK_LEVEL EXPECTED_NATIVE_FINGERPRINT\n",
                argv[0]);
        return 2;
    }

    root = argv[1];
    dbpath = argv[2];
    gpgdir = argv[3];
    package_path = argv[4];
    package_check_level = argv[5];
    database_check_level = argv[6];
    expected_fingerprint = argv[7];

    if (!path_is_absolute(root) || !path_is_absolute(dbpath)
            || !path_is_absolute(gpgdir) || !path_is_absolute(package_path)) {
        fprintf(stderr, "ROOT, DBPATH, GPGDIR, and PACKAGE must be absolute paths\n");
        return 2;
    }
    if (strcmp(package_check_level, "required") == 0) {
        package_siglevel = ALPM_SIG_PACKAGE;
    } else if (strcmp(package_check_level, "disabled") == 0) {
        package_siglevel = 0;
    } else {
        fprintf(stderr, "PACKAGE_CHECK_LEVEL must be required or disabled\n");
        return 2;
    }
    if (strcmp(database_check_level, "required") == 0) {
        requested_database_siglevel = ALPM_SIG_DATABASE;
    } else if (strcmp(database_check_level, "disabled") == 0) {
        requested_database_siglevel = 0;
    } else {
        fprintf(stderr, "DATABASE_CHECK_LEVEL must be required or disabled\n");
        return 2;
    }
    if (expected_fingerprint[0] == '\0') {
        fprintf(stderr, "EXPECTED_NATIVE_FINGERPRINT must not be empty\n");
        return 2;
    }

    handle = alpm_initialize(root, dbpath, &initialize_error);
    if (handle == NULL) {
        fprintf(stderr, "alpm_initialize: %s (%d)\n",
                alpm_strerror(initialize_error), (int)initialize_error);
        goto cleanup;
    }

    if (alpm_option_set_questioncb(handle, refuse_questions, &questions) != 0) {
        report_alpm_error(handle, "alpm_option_set_questioncb");
        goto cleanup;
    }
    if (alpm_option_set_gpgdir(handle, gpgdir) != 0) {
        report_alpm_error(handle, "alpm_option_set_gpgdir");
        goto cleanup;
    }

    package_load_return = alpm_pkg_load(handle,
            package_path,
            1,
            package_siglevel,
            &file_package);
    package_errno_ok = alpm_errno(handle) == ALPM_ERR_OK;
    package_non_null = file_package != NULL;
    if (package_load_return != 0 || !package_errno_ok || !package_non_null) {
        fprintf(stderr,
                "alpm_pkg_load returned %d, errno_ok=%d, package_non_null=%d: %s\n",
                package_load_return,
                package_errno_ok,
                package_non_null,
                alpm_strerror(alpm_errno(handle)));
        goto cleanup;
    }

    package_signature_initialized = 1;
    if (capture_signature("package",
                alpm_pkg_check_pgp_signature(file_package, &package_signature_list),
                &package_signature_list,
                expected_fingerprint,
                &package_signature) != 0) {
        report_alpm_error(handle, "alpm_pkg_check_pgp_signature");
        goto cleanup;
    }

    database = alpm_register_syncdb(handle,
            REPOSITORY_NAME,
            requested_database_siglevel);
    database_non_null = database != NULL;
    if (!database_non_null) {
        report_alpm_error(handle, "alpm_register_syncdb");
        goto cleanup;
    }

    database_siglevel = alpm_db_get_siglevel(database);
    if (database_siglevel != requested_database_siglevel) {
        fprintf(stderr, "database siglevel was %d, expected %d\n",
                database_siglevel, requested_database_siglevel);
        goto cleanup;
    }

    database_valid_return = alpm_db_get_valid(database);
    if (database_valid_return != 0) {
        report_alpm_error(handle, "alpm_db_get_valid");
        goto cleanup;
    }

    database_signature_initialized = 1;
    if (capture_signature("database",
                alpm_db_check_pgp_signature(database, &database_signature_list),
                &database_signature_list,
                expected_fingerprint,
                &database_signature) != 0) {
        report_alpm_error(handle, "alpm_db_check_pgp_signature");
        goto cleanup;
    }

    database_package = alpm_db_get_pkg(database, PACKAGE_NAME);
    if (database_package == NULL) {
        report_alpm_error(handle, "alpm_db_get_pkg");
        goto cleanup;
    }

    {
        const char *name = alpm_pkg_get_name(database_package);
        const char *version = alpm_pkg_get_version(database_package);
        const char *filename = alpm_pkg_get_filename(database_package);

        if (name == NULL || version == NULL || filename == NULL) {
            fprintf(stderr, "database package has a null name, version, or filename\n");
            goto cleanup;
        }

        database_package_name = strdup(name);
        database_package_version = strdup(version);
        database_package_filename = strdup(filename);
        if (database_package_name == NULL || database_package_version == NULL
                || database_package_filename == NULL) {
            fprintf(stderr, "out of memory while copying database package metadata\n");
            goto cleanup;
        }
    }

    if (strcmp(database_package_name, PACKAGE_NAME) != 0
            || strcmp(database_package_version, PACKAGE_VERSION) != 0
            || strcmp(database_package_filename, PACKAGE_FILENAME) != 0) {
        fprintf(stderr,
                "database package was %s %s %s; expected %s %s %s\n",
                database_package_name,
                database_package_version,
                database_package_filename,
                PACKAGE_NAME,
                PACKAGE_VERSION,
                PACKAGE_FILENAME);
        goto cleanup;
    }

    if (questions.calls != 0) {
        fprintf(stderr, "question callback was invoked %zu times\n", questions.calls);
        goto cleanup;
    }

    candidate_ready = 1;

cleanup:
    if (package_signature_initialized
            && alpm_siglist_cleanup(&package_signature_list) != 0) {
        fprintf(stderr, "alpm_siglist_cleanup(package) failed\n");
        cleanup_ok = 0;
    }
    if (database_signature_initialized
            && alpm_siglist_cleanup(&database_signature_list) != 0) {
        fprintf(stderr, "alpm_siglist_cleanup(database) failed\n");
        cleanup_ok = 0;
    }
    if (file_package != NULL && alpm_pkg_free(file_package) != 0) {
        fprintf(stderr, "alpm_pkg_free failed\n");
        cleanup_ok = 0;
    }
    if (handle != NULL && alpm_release(handle) != 0) {
        fprintf(stderr, "alpm_release failed\n");
        cleanup_ok = 0;
    }

    if (!candidate_ready || !cleanup_ok) {
        fprintf(stderr, "native probe failed; question callback calls=%zu\n", questions.calls);
        free(package_signature.fingerprint);
        free(database_signature.fingerprint);
        free(database_package_name);
        free(database_package_version);
        free(database_package_filename);
        return 1;
    }

    fputs("{\"schema\":\"" OUTPUT_SCHEMA "\",\"question_callback_calls\":", stdout);
    printf("%zu", questions.calls);
    fputs(",\"package\":{\"configured_siglevel\":", stdout);
    printf("%d", package_siglevel);
    fputs(",\"full_archive_requested\":true,\"load_return\":", stdout);
    printf("%d", package_load_return);
    fputs(",\"errno_ok\":true,\"package_non_null\":true,\"signature\":{\"check_return\":", stdout);
    printf("%d", package_signature.check_return);
    fputs(",\"count\":", stdout);
    printf("%zu", package_signature.count);
    fputs(",\"fingerprint\":", stdout);
    write_json_string(package_signature.fingerprint);
    fputs(",\"status\":", stdout);
    write_json_string(status_name(package_signature.status));
    fputs(",\"validity\":", stdout);
    write_json_string(validity_name(package_signature.validity));
    fputs("}},\"database\":{\"configured_siglevel\":", stdout);
    printf("%d", database_siglevel);
    fputs(",\"database_non_null\":true,\"valid_return\":", stdout);
    printf("%d", database_valid_return);
    fputs(",\"signature\":{\"check_return\":", stdout);
    printf("%d", database_signature.check_return);
    fputs(",\"count\":", stdout);
    printf("%zu", database_signature.count);
    fputs(",\"fingerprint\":", stdout);
    write_json_string(database_signature.fingerprint);
    fputs(",\"status\":", stdout);
    write_json_string(status_name(database_signature.status));
    fputs(",\"validity\":", stdout);
    write_json_string(validity_name(database_signature.validity));
    fputs("},\"package\":{\"name\":", stdout);
    write_json_string(database_package_name);
    fputs(",\"version\":", stdout);
    write_json_string(database_package_version);
    fputs(",\"filename\":", stdout);
    write_json_string(database_package_filename);
    fputs("}}}\n", stdout);

    free(package_signature.fingerprint);
    free(database_signature.fingerprint);
    free(database_package_name);
    free(database_package_version);
    free(database_package_filename);

    if (fflush(stdout) != 0 || ferror(stdout)) {
        fprintf(stderr, "failed to write probe evidence\n");
        return 1;
    }
    return 0;
}
