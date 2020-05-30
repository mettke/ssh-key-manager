DELIMITER ;;
CREATE FUNCTION UUID_TO_BIN(uuid CHAR(36), swap_flag TINYINT)
RETURNS BINARY(16)
BEGIN
    SET @bin = UNHEX(REPLACE(uuid, '-', ''));
    IF swap_flag > 0  THEN
        SET @bin_tmp = @bin;
        SET @bin = INSERT(@bin, 1, 2, SUBSTRING(@bin_tmp, 7, 2));
        SET @bin = INSERT(@bin, 3, 2, SUBSTRING(@bin_tmp, 5, 2));
        SET @bin = INSERT(@bin, 5, 2, SUBSTRING(@bin_tmp, 1, 2));
        SET @bin = INSERT(@bin, 7, 2, SUBSTRING(@bin_tmp, 3, 2));
    END IF;
    RETURN @bin;
END;;
DELIMITER ;
