// Remember the last-selected tab in a tab group
$(function() {
    if (sessionStorage) {
        $('a[data-toggle="tab"]').on('shown.bs.tab', function(e) {
            //save the latest tab
            sessionStorage.setItem('lastTab' + location.pathname, $(e.target).attr('href'));
        });

        //go to the latest tab, if it exists:
        var lastTab = sessionStorage.getItem('lastTab' + location.pathname);

        if (lastTab) {
            $('a[href="' + lastTab + '"]').tab('show');
        } else {
            $('a[data-toggle="tab"]:first').tab('show');
        }
    }

    get_tab_from_location();
    window.onpopstate = function(event) {
        get_tab_from_location();
    }

    function get_tab_from_location() {
        // Javascript to enable link to tab
        var url = document.location.toString();
        if (url.match('#')) {
            $('.nav-tabs a[href="#' + url.split('#')[1] + '"]').tab('show');
        }
    }

    // Do the location modifying code after all other setup, since we don't want the initial loading to trigger this
    $('a[data-toggle="tab"]').on('shown.bs.tab', function(e) {
        if (history) {
            history.replaceState(null, null, e.target.href);
        } else {
            window.location.hash = e.target.hash;
        }
    });
});

// Remember the expanded-state of a collapsible section
$(function() {
    get_section_from_location();
    window.onpopstate = function(event) {
        get_section_from_location();
    }

    function get_section_from_location() {
        // Javascript to enable link to section
        var url = document.location.toString();
        if (url.match('#')) {
            var fragment = url.split('#')[1];
        } else {
            var fragment = '';
        }
        $(".collapse").each(function() {
            if (this.id == fragment) $(this).addClass("in");
            else $(this).removeClass("in");
        });
    }

    // Do the location modifying code after all other setup, since we don't want the initial loading to trigger this
    $('.panel-collapse').on('show.bs.collapse', function(e) {
        if (history) {
            history.replaceState(null, null, '#' + e.target.id);
        } else {
            window.location.hash = e.target.id;
        }
    });

});

// Show only chosen fingerprint hash format in list views
$(function() {
    $('table th.fingerprint').first().each(function() {
        $(this).append(' ');
        var select = $('<select>');
        var options = ['MD5', 'SHA256'];
        for (var i = 0, option; option = options[i]; i++) {
            select.append($('<option>').text(option).val(option));
        }
        if (localStorage) {
            var fingerprint_hash = localStorage.getItem('preferred_fingerprint_hash');
            if (fingerprint_hash) {
                select.val(fingerprint_hash);
            }
        }
        $(this).append(select);
        select.on('change', function() {
            if (this.value == 'SHA256') {
                $('span.fingerprint_md5').hide();
                $('span.fingerprint_sha256').show();
            } else {
                $('span.fingerprint_sha256').hide();
                $('span.fingerprint_md5').show();
            }
            if (localStorage) {
                localStorage.setItem('preferred_fingerprint_hash', this.value);
            }
        });
    });
});

// Add confirmation dialog to all submit buttons with data-confirm attribute
$(function() {
    $('button[type="submit"][data-confirm]').each(function() {
        $(this).on('click', function() { return confirm($(this).data('confirm')); });
    });
});

// Add "clear field" button functionality
$(function() {
    $('button[data-clear]').each(function() {
        $(this).on('click', function() { this.form[$(this).data('clear')].value = ''; });
    });
});

// Home page dynamic add pubkey form
$(function() {
    $('#add_key_button').on('click', function() {
        $('#help').hide().removeClass('hidden');
        $('#add_key_form').hide().removeClass('hidden');
        $('#add_key_form').show('fast');
        $('#add_key_button').hide();
        $('#add_public_key').focus();
    });
    $('#add_key_form button[type=button].btn-info').on('click', function() {
        $('#help').toggle('fast');
    });
    $('#add_key_form button[type=button].btn-default').on('click', function() {
        $('#add_key_form').hide('fast');
        $('#add_key_button').show();
    });
});

// Server sync status
$(function() {
    var status_div = $('#server_sync_status');
    status_div.each(function() {
        if (status_div.data('class')) {
            update_server_sync_status(status_div.data('class'), status_div.data('message'));
            $('span.server_account_sync_status').each(function() {
                update_server_account_sync_status(this.id, $(this).data('class'), $(this).data('message'));
            });
        } else {
            $('span', status_div).addClass('text-warning');
            $('span', status_div).text('Pending');
            $('span.server_account_sync_status').addClass('text-warning');
            $('span.server_account_sync_status').text('Pending');
            var timeout = 1000;
            var max_timeout = 10000;
            get_server_sync_status();
        }

        function get_server_sync_status() {
            var xhr = $.ajax({
                url: window.location.pathname + '/sync_status',
                dataType: 'json'
            });
            xhr.done(function(status) {
                if (status.pending) {
                    timeout = Math.min(timeout * 1.5, max_timeout);
                    setTimeout(get_server_sync_status, timeout);
                } else {
                    var classname;
                    if (status.sync_status == 'sync success') classname = 'success';
                    if (status.sync_status == 'sync failure') classname = 'danger';
                    if (status.sync_status == 'sync warning') classname = 'warning';
                    update_server_sync_status(classname, status.last_sync.details);
                }
                $.each(status.accounts, function(index, item) {
                    if (!item.pending) {
                        var classname;
                        var message;
                        if (item.sync_status == 'proposed') { classname = 'info';
                            message = 'Requested'; }
                        if (item.sync_status == 'sync success') { classname = 'success';
                            message = 'Synced'; }
                        if (item.sync_status == 'sync failure') { classname = 'danger';
                            message = 'Failed'; }
                        if (item.sync_status == 'sync warning') { classname = 'warning';
                            message = 'Not synced'; }
                        update_server_account_sync_status('server_account_sync_status_' + item.name, classname, message);
                    }
                });
            });
        }

        function update_server_sync_status(classname, message) {
            $('span', status_div).removeClass('text-success text-warning text-danger');
            $('span', status_div).addClass('text-' + classname);
            $('span', status_div).text(message);
            if (classname == 'success') {
                $('a', status_div).addClass('hidden');
            } else {
                $('a', status_div).removeClass('hidden');
                let path = $('a', status_div).prop('href');
                if (path.indexOf('#') > 0) {
                    path = path.substring(0, path.indexOf('#'));
                }
                if (classname == 'warning') $('a', status_div).prop('href', path + '#sync_warning');
                if (classname == 'danger') $('a', status_div).prop('href', path + '#sync_error');
            }
            $('div.spinner', status_div).remove();
            $('button[name=sync]', status_div).removeClass('invisible');
        }

        function update_server_account_sync_status(id, classname, message) {
            $('#' + id).removeClass('text-success text-warning text-danger');
            $('#' + id).addClass('text-' + classname);
            $('#' + id).text(message);
        }
    });
});